# Nghiên cứu Seam Kỹ thuật Windows Resident Access & Settings (DropCove 0.0.4)

Tài liệu này xác định các Win32 API, Tauri v2 subsystem và Rust crate seams cần thiết để hiện thực Resident access và Settings cho DropCove 0.0.4 trên Windows 11 x64 MSVC, dựa trên các tài liệu đặc tả chuẩn (`docs/SPECIFICATION.md` mục 5, 7.1, 16, 18 và `docs/ARCHITECTURE.md` mục 2.1, 2.5, 11, 12).

---

## 1. Profile-Scoped Single Instance và Activation Signaling

### 1.1 Yêu cầu chuẩn (Contract Requirements)
- Chạy đúng 1 Resident Instance cho mỗi DropCove profile đã resolve; profile mặc định nằm trong Windows user profile tại `%LOCALAPPDATA%\DropCove`.
- Lần khởi chạy thứ 2 (second launch) của cùng profile:
  1. Phát tín hiệu kích hoạt kèm tham số/môi trường tới instance đang chạy.
  2. Thoát ngay lập tức (`exit code 0`).
  3. Instance đang chạy nhận tín hiệu, hiển thị và kích hoạt Drop Shelf lên foreground tại monitor chứa con trỏ chuột.

### 1.2 Nền tảng Win32 & Cơ chế IPC
Trên Windows, cơ chế single-instance tiêu chuẩn kết hợp:
1. **Named Mutex (`CreateMutexW`)**:
   - Tên mutex phải nằm trong session scope của user (`Local\<Identifier>-sim` hoặc mặc định session namespace `Local\`). Khi đặt tên không có prefix `Global\`, đối tượng tự động gắn với Win32 session hiện tại của người dùng đăng nhập.
   - Khi tiến trình thứ hai gọi `CreateMutexW(NULL, TRUE, name)` và `GetLastError() == ERROR_ALREADY_EXISTS`, tiến trình phát hiện đã có instance chạy trước.
2. **HWND Rendezvous & Data Forwarding (`WM_COPYDATA` & `SendMessageTimeoutW`)**:
   - Instance chạy đầu tiên tạo một message-only window hoặc hidden event target window với class name và window name duy nhất (`<id>-sic`, `<id>-siw`).
   - Instance thứ hai tìm HWND này qua `FindWindowW(class_name, window_name)`.
   - Instance thứ hai gửi dữ liệu dòng lệnh (`args`, `cwd`) qua cấu trúc `COPYDATASTRUCT` với thông điệp `WM_COPYDATA` bằng `SendMessageTimeoutW` (tránh block vô hạn nếu instance đầu bị treo).
3. **Chuyển giao quyền Foreground Focus (`AllowSetForegroundWindow`)**:
   - Windows áp dụng cơ chế Foreground Lock để chống focus-stealing: một tiến trình nền không thể tự ý gọi `SetForegroundWindow` để cướp focus nếu người dùng đang thao tác ở cửa sổ khác.
   - Tuy nhiên, tiến trình thứ hai (vốn được người dùng vừa kích hoạt từ Explorer, Start menu hoặc Shortcut) có quyền foreground. Trước khi gửi message hoặc thoát, instance thứ hai gọi:
     ```cpp
     DWORD target_pid = 0;
     GetWindowThreadProcessId(hwnd, &target_pid);
     AllowSetForegroundWindow(target_pid);
     ```
   - Lệnh này cấp quyền hợp lệ cho instance đầu tiên được phép gọi `SetForegroundWindow` để đưa Drop Shelf lên trước mặt người dùng.

### 1.3 Đánh giá Plugin Tauri: `tauri-plugin-single-instance`
- **Mã nguồn thẩm định:** Plugin chính thức `tauri-plugin-single-instance` (v2.x trong `plugins-workspace`, file `src/platform_impl/windows.rs`).
- **Cơ chế hoạt động:**
  - Setup plugin kiểm tra `CreateMutexW(NULL, TRUE, format!("{id}-sim"))`.
  - Nếu `ERROR_ALREADY_EXISTS`:
    - Lấy HWND qua `FindWindowW`.
    - Lấy PID qua `GetWindowThreadProcessId`.
    - Gọi `AllowSetForegroundWindow(pid)`.
    - Thu thập `std::env::current_dir()` và `std::env::args_os()`, đóng gói dạng `cwd|arg1|arg2...`.
    - Gửi qua `WM_COPYDATA` bằng `SendMessageTimeoutW` (timeout 10,000 ms).
    - Gọi `app.cleanup_before_exit()` và `std::process::exit(0)`.
  - Nếu là instance đầu:
    - Tạo hidden event target window `create_event_target_window` với styles `WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW` và gán window proc để nhận `WM_COPYDATA`.
    - Khi nhận `WM_COPYDATA`, parse `(args, cwd)` và gọi callback đăng ký với `AppHandle`.
- **Độ tin cậy:** Plugin phản ánh pattern Win32 đã được kiểm chứng, nhưng identity mặc định theo application identifier chưa tự đáp ứng custom profile scope của DropCove. Delivery phải truyền/triển khai identity dẫn xuất từ resolved profile hoặc dùng adapter native tương đương; không được coi plugin mặc định là profile-scoped.

### 1.4 Primary Sources
- Microsoft Learn: `CreateMutexW` - <https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw>
- Microsoft Learn: `WM_COPYDATA` message - <https://learn.microsoft.com/en-us/windows/win32/dataxchg/wm-copydata>
- Microsoft Learn: `AllowSetForegroundWindow` - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-allowsetforegroundwindow>
- Tauri Docs: Single Instance Plugin - <https://v2.tauri.app/plugin/single-instance/>
- Tauri plugins-workspace: `plugins/single-instance/src/platform_impl/windows.rs` - <https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/single-instance/src/platform_impl/windows.rs>

---

## 2. Notification Area (Tray Icon), Menu và Thông báo

### 2.1 Yêu cầu chuẩn
- Tray icon luôn hiện diện trong suốt vòng đời resident process.
- Chuột trái (Left click) hoặc nhấp đúp (Double click): Hiển thị và kích hoạt Drop Shelf tại monitor chứa con trỏ.
- Chuột phải (Right click): Mở context menu chứa **Settings** và **Exit DropCove**.
- **Exit DropCove**: Kết thúc resident process sạch sẽ.
- Khi đóng Drop Shelf: Không thoát tiến trình resident.
- Hiển thị thông báo (warning/recovery notification) khi xảy ra lỗi khôi phục database hoặc đăng ký hotkey thất bại.

### 2.2 Tích hợp Tray trong Tauri v2 Core
- Crate `tauri` hiện tại trong `Cargo.toml` đã bật sẵn feature `"tray-icon"`.
- Tauri v2 quản lý native Win32 tray qua module `tauri::tray`:
  - `TrayIconBuilder::new()`: Thiết lập id, icon, tooltip, menu, và event handler.
  - Sự kiện click chuột: Nhận diện qua callback `.on_tray_icon_event(|tray, event| ...)`:
    - `TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }`: Trigger hiển thị và kích hoạt Drop Shelf.
    - `TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }`: Xử lý tương tự (debounce nếu cần).
  - Context menu chuột phải:
    - Xây dựng qua `tauri::menu::MenuBuilder::new(app)`:
      - Item 1: `MenuItem::with_id(app, "settings", "Settings", true, None)`
      - Item 2: `MenuItem::with_id(app, "exit", "Exit DropCove", true, None)`
    - Gắn vào tray qua `.menu(&tray_menu)`. Trên Windows, click chuột phải sẽ tự động popup native Win32 menu theo tiêu chuẩn Shell_NotifyIcon.
    - Callback `.on_menu_event(|app, event| ...)` bắt id:
      - `"settings"`: Mở cửa sổ Settings (hoặc focus nếu đã mở).
      - `"exit"`: Gọi `app.exit(0)` kết thúc resident process.

### 2.3 Cảnh báo & Thông báo hệ thống
- Tùy chọn 1: Dùng plugin chính thức `tauri-plugin-notification` (`Notification::new().title(...).body(...).show()`).
- Tùy chọn 2 (Native balloon / toast qua `Shell_NotifyIconW` với cờ `NIF_INFO` trong Win32 SDK): Nếu muốn tránh thêm plugin dependency, có thể gọi Win32 Shell API hoặc dùng balloon notification của notification-area icon có sẵn. Tuy nhiên, `tauri-plugin-notification` trên Windows 11 map trực tiếp vào Windows Action Center / Toast notifications chuẩn của OS.

### 2.4 Primary Sources
- Tauri v2 `tauri::tray::TrayIconBuilder` - <https://docs.rs/tauri/latest/tauri/tray/struct.TrayIconBuilder.html>
- Tauri v2 `tauri::tray::TrayIconEvent` - <https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html>
- Tauri v2 `tauri::menu::MenuBuilder` - <https://docs.rs/tauri/latest/tauri/menu/struct.MenuBuilder.html>
- Microsoft Learn: `Shell_NotifyIconW` - <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw>
- Tauri Docs: Notification Plugin - <https://v2.tauri.app/plugin/notification/>

---

## 3. Global Hotkey: Win32 RegisterHotKey & MOD_NOREPEAT

### 3.1 Yêu cầu chuẩn & Default Shortcut
- Mặc định: `Ctrl + Shift + Space`.
- Phím tắt có thể cấu hình lại trong Settings: cần ít nhất một modifier (`Ctrl`, `Alt`, `Shift`, `Win`) cộng với một virtual key hợp lệ.
- Thao tác hotkey: Toggle hoặc hiển thị Drop Shelf tại monitor chứa con trỏ.
- Nếu đăng ký thất bại lúc startup (bị app khác chiếm): Giữ resident process tiếp tục chạy và báo warning notification.

### 3.2 Cơ chế Win32 Native: `RegisterHotKey`
Hàm Win32 API:
```cpp
BOOL RegisterHotKey(
  [in, optional] HWND hWnd,
  [in]           int  id,
  [in]           UINT fsModifiers,
  [in]           UINT vk
);
```

#### Các ràng buộc hệ thống cốt lõi:
1. **Thread Message Loop Constraint:**
   - Nếu `hWnd != NULL`, cửa sổ nhận thông điệp `WM_HOTKEY` phải được tạo bởi chính thread gọi `RegisterHotKey`. Windows cấm liên kết hotkey với HWND được tạo bởi thread khác.
   - Nếu `hWnd == NULL`, `WM_HOTKEY` được đẩy vào thread message queue của thread gọi hàm, bắt buộc thread đó phải chạy vòng lặp message pump (`GetMessage` / `DispatchMessage`).
2. **Xung đột Hotkey (`ERROR_HOTKEY_ALREADY_REGISTERED` = 1418):**
   - Nếu tổ hợp modifier + VK đã bị ứng dụng khác (hoặc chính OS) chiếm dụng, `RegisterHotKey` trả về `0` và `GetLastError()` trả về `1418`.
   - Phím `F12` bị Windows bảo lưu vĩnh viễn cho debugger (kernel/JIT debugger) và không bao giờ được phép đăng ký.
3. **Cờ chống lặp phím `MOD_NOREPEAT` (0x4000):**
   - Theo tài liệu Microsoft Learn, nếu không truyền `MOD_NOREPEAT`, khi người dùng nhấn giữ tổ hợp phím, tính năng keyboard auto-repeat của phần cứng/OS sẽ liên tục bắn các thông điệp `WM_HOTKEY` dồn dập vào message loop.
   - Khi cờ `MOD_NOREPEAT` được chỉ định: Hệ thống chỉ sinh duy nhất 1 thông điệp `WM_HOTKEY` khi phím được nhấn xuống. Người dùng bắt buộc phải thả phím chính ra và nhấn lại thì mới sinh event tiếp theo. Điều này ngăn ngừa hoàn toàn hiện tượng nhấp nháy cửa sổ (shelf toggle storm) khi người dùng tì tay lên phím.

### 3.3 Đánh giá Triển khai: `tauri-plugin-global-shortcut` vs `windows` crate
- Thẩm định mã nguồn `tauri-apps/global-hotkey` (crate tầng dưới của `tauri-plugin-global-shortcut`):
  - File `src/platform_impl/windows/mod.rs` cho thấy crate này:
    1. Tạo một hidden background window với class riêng.
    2. Trong hàm `register`: Thiết lập `let mut mods = MOD_NOREPEAT;` làm cờ cơ sở.
    3. Hỗ trợ đầy đủ `MOD_SHIFT`, `MOD_WIN`, `MOD_ALT`, `MOD_CONTROL`.
    4. Xử lý chính xác mã lỗi `ERROR_HOTKEY_ALREADY_REGISTERED`.
- **Khuyến nghị kiến trúc:** Sử dụng `tauri-plugin-global-shortcut` hoặc gọi trực tiếp `windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey` trên một message window của DropCove. Cả hai giải pháp đều đảm bảo tuyệt đối cờ `MOD_NOREPEAT`.

### 3.4 Primary Sources
- Microsoft Learn: `RegisterHotKey` function - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey>
- Microsoft Learn: `WM_HOTKEY` message - <https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-hotkey>
- Crate `global-hotkey` source (`src/platform_impl/windows/mod.rs`) - <https://github.com/tauri-apps/global-hotkey/blob/dev/src/platform_impl/windows/mod.rs>

---

## 4. Sign-in Startup Registration: HKCU Run & Quoting `--autostart`

### 4.1 Yêu cầu chuẩn
- Cấu hình **Start with Windows** được bật mặc định.
- Lưu tại Registry key cấp user: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.
- Định dạng chuỗi giá trị (Value data):
  ```text
  "C:\Path\To\DropCove.exe" --autostart
  ```
- Ứng dụng DropCove trực tiếp sở hữu và quản lý mục đăng ký này; bộ cài NSIS không tự ý bật/tắt độc lập.
- Khởi động ẩn (hidden start) khi có cờ `--autostart`: Không tự động hiển thị Drop Shelf hoặc Edge Rail.

### 4.2 Win32 Registry Contract & Quy tắc Command Line
1. **Khóa Registry:**
   - Key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
   - Value Name: `"DropCove"`
   - Value Type: `REG_SZ`
2. **Quy tắc Quoting đường dẫn:**
   - Theo tài liệu Microsoft Learn (*Run and RunOnce Registry Keys* & *Parsing C++ Command-Line Arguments*): Nếu đường dẫn file thực thi chứa khoảng trắng (ví dụ `%LOCALAPPDATA%\Programs\DropCove\DropCove.exe` với username có space như `C:\Users\John Doe\...`) mà không được bao bọc trong dấu ngoặc kép (`"`), Windows Shell sẽ ngắt chuỗi tại khoảng trắng đầu tiên và báo lỗi file không tìm thấy hoặc gây ra lỗ hổng Unquoted Search Path.
   - Do đó, đường dẫn thực thi **bắt buộc phải đặt trong dấu ngoặc kép đầy đủ**, theo sau bởi một khoảng trắng và cờ `--autostart`:
     ```text
     "\"C:\\Users\\User Name\\AppData\\Local\\Programs\\DropCove\\DropCove.exe\" --autostart"
     ```
3. **Task Manager Startup Impact:**
   - Trên Windows 10/11, Task Manager theo dõi trạng thái disable autostart tại `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`. Nếu ghi registry thông thường mà key `StartupApproved` bị user vô hiệu hóa từ Task Manager, ứng dụng vẫn sẽ bị chặn. Crate `auto-launch` (được dùng bên dưới `tauri-plugin-autostart`) quản lý tự động cả 2 khóa này.

### 4.3 Đánh giá Plugin: `tauri-plugin-autostart`
- Crate `tauri-plugin-autostart` (phiên bản `2.x`) cung cấp API `enable()`, `disable()`, `is_enabled()`.
- Cấu hình tham số dòng lệnh qua builder:
  ```rust
  tauri_plugin_autostart::Builder::new()
      .arg("--autostart")
      .app_name("DropCove")
      .build()
  ```
- Thẩm định mã nguồn `auto-launch/src/windows_command_line.rs`: Hàm `build_command_line` định dạng chính xác `"{app_path}" --autostart`, bao bọc đường dẫn bằng dấu ngoặc kép và xử lý escape backslash chuẩn C runtime.

### 4.4 Primary Sources
- Microsoft Learn: `Run and RunOnce Registry Keys` - <https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys>
- Microsoft Learn: `Parsing C++ Command-Line Arguments` - <https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments>
- Tauri Docs: Autostart Plugin - <https://v2.tauri.app/plugin/autostart/>
- Crate `auto-launch` Windows command line parser - <https://github.com/zzzgydi/auto-launch/blob/master/src/windows_command_line.rs>

---

## 5. Cursor-Monitor Placement & Per-Monitor V2 DPI

### 5.1 Yêu cầu chuẩn
- Kích thước quy định theo logical pixels (ví dụ shelf gốc `180 × 180`).
- Toàn bộ bounds cửa sổ vật lý (physical window bounds) phải nhân tỉ lệ tương ứng với DPI của monitor mục tiêu:
  $$\text{physical} = \text{logical} \times \frac{\text{DPI}}{96}$$
- Điểm hiển thị khi kích hoạt bằng Normal launch, Second launch, Hotkey, Tray click, Shake: Xác định **monitor đang chứa con trỏ chuột** tại thời điểm kích hoạt.
- Cửa sổ phải được clamp nằm gọn trong Work Area (trừ Taskbar) của monitor đó.
- Quy trình Per-Monitor V2 DPI-aware: Khi di chuyển qua lại giữa các monitor có tỉ lệ scale khác nhau, cửa sổ phải reflow kích thước mượt mà.

### 5.2 Win32 APIs xác định Cursor Monitor & Work Area
1. **Lấy vị trí con trỏ chuột hiện tại:**
   - `GetCursorPos(&mut pt)`: Lấy tọa độ ảo toàn màn hình (virtual screen coordinates) theo physical pixels.
2. **Xác định Monitor từ tọa độ:**
   - `MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST)`: Trả về `HMONITOR` chứa con trỏ, hoặc monitor gần nhất nếu con trỏ nằm ngoài bounds hợp lệ.
3. **Truy vấn thông số Monitor & Work Area:**
   - Dùng `GetMonitorInfoW(hmonitor, &mut mi)` với cấu trúc `MONITORINFO`:
     - `mi.rcMonitor`: Toàn bộ kích thước vật lý của màn hình.
     - `mi.rcWork`: Vùng làm việc khả dụng (đã trừ Taskbar và docked appbars).
4. **Truy vấn DPI của Monitor:**
   - Windows 10/11: `GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y)` (từ thư viện `Shcore.dll` / `Win32_UI_HiDpi`).
   - Hoặc khi đã có HWND: `GetDpiForWindow(hwnd)`.

### 5.3 Per-Monitor V2 DPI Scaling trong Tauri v2
- Tauri v2 tự động khai báo Per-Monitor V2 DPI awareness trong manifest ứng dụng trên Windows (`dpiAwareness = PerMonitorV2`).
- Trong runtime của Tauri, các API `Window::set_position` và `Window::set_size` nhận kiểu `tauri::Position` và `tauri::Size` hỗ trợ cả `LogicalPosition`/`LogicalSize` lẫn `PhysicalPosition`/`PhysicalSize`.
- Để định vị chính xác cửa sổ theo monitor con trỏ:
  1. Lấy `HMONITOR` và `rcWork` từ Win32 API.
  2. Tính tọa độ mong muốn (ví dụ: căn giữa hoặc đặt gần con trỏ).
  3. Clamp bounding box sao cho:
     $$X \ge \text{rcWork.left}, \quad X + \text{width} \le \text{rcWork.right}$$
     $$Y \ge \text{rcWork.top}, \quad Y + \text{height} \le \text{rcWork.bottom}$$
  4. Đặt vị trí cửa sổ qua `window.set_position(PhysicalPosition::new(x, y))` hoặc qua Win32 `SetWindowPos`.

### 5.4 Primary Sources
- Microsoft Learn: `MonitorFromPoint` - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-monitorfrompoint>
- Microsoft Learn: `MONITORINFO` structure - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfo>
- Microsoft Learn: `GetDpiForMonitor` - <https://learn.microsoft.com/en-us/windows/win32/api/shellscalingapi/nf-shellscalingapi-getdpiformonitor>
- Microsoft Learn: High DPI Desktop Application Development on Windows - <https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows>

---

## 6. Foreground Activation Limitations & Topmost Repair

### 6.1 Giới hạn Kích hoạt Foreground trên Windows
- Hệ thống Windows áp dụng chính sách bảo vệ Foreground nghiêm ngặt:
  - Một tiến trình nền không thể tự ý gọi `SetForegroundWindow` trừ khi thỏa mãn một trong các điều kiện:
    - Vừa nhận input từ người dùng (như hotkey hoặc click tray).
    - Được tiến trình foreground khác ủy quyền qua `AllowSetForegroundWindow`.
    - Không có tiến trình nào đang chiếm foreground.
  - Nếu vi phạm, Windows từ chối focus và chỉ nhấp nháy icon dưới taskbar.
- Đối với DropCove:
  - Kích hoạt qua Hotkey hoặc Tray click: Bản thân hành động nhấn phím/click là user input trực tiếp, Windows cho phép DropCove active lên foreground hợp lệ.
  - Kích hoạt qua Second launch: Instance 2 phải gọi `AllowSetForegroundWindow(pid_instance_1)` trước khi thoát.

### 6.2 Hiện tượng Trôi Z-Order (Topmost Degradation) & Cơ chế Topmost Repair
- **Vấn đề thực tế của Windows Desktop:**
  - Cửa sổ Drop Shelf được cấu hình thuộc tính `alwaysOnTop: true` (`WS_EX_TOPMOST`).
  - Tuy nhiên, trong môi trường Windows, khi có một ứng dụng topmost khác xuất hiện (ví dụ Task Manager, công cụ chụp màn hình, game không viền, hoặc notification banner), z-order của Drop Shelf có thể bị đẩy xuống phía dưới cửa sổ topmost mới đó.
  - Mất focus không làm ẩn Drop Shelf, nhưng người dùng mong đợi Drop Shelf duy trì trạng thái topmost tương đối khi họ thao tác qua lại giữa các cửa sổ thông thường.
- **Topmost Repair Mechanism (`SetWindowPos`):**
  - Để đưa cửa sổ trở lại đỉnh z-order mà **không cướp focus nhập liệu** của người dùng (no focus stealing):
    ```cpp
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0, 0, 0, 0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE
    );
    ```
  - Cờ `SWP_NOACTIVATE` cực kỳ quan trọng: Nó đảm bảo z-order được cập nhật lên đầu nhóm topmost mà không làm mất focus bàn phím của ứng dụng người dùng đang soạn thảo văn bản.
- **Theo dõi thay đổi Foreground (`SetWinEventHook`):**
  - Architecture §12 quy định: "Drop Shelf tracker — while the shelf is visible, foreground changes reassert its topmost z-order without activating it."
  - Để lắng nghe sự kiện cửa sổ foreground hệ thống thay đổi mà không phải dùng polling:
    ```cpp
    HWINEVENTHOOK hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_FOREGROUND,
        NULL,
        WinEventProc,
        0, 0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
    );
    ```
  - Khi nhận `EVENT_SYSTEM_FOREGROUND`: Kiểm tra nếu Drop Shelf đang visible, gọi `SetWindowPos(hwnd, HWND_TOPMOST, ..., SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE)`.

### 6.3 Primary Sources
- Microsoft Learn: `SetForegroundWindow` - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow>
- Microsoft Learn: `SetWindowPos` (HWND_TOPMOST, SWP_NOACTIVATE) - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos>
- Microsoft Learn: `SetWinEventHook` - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook>

---

## 7. Atomic Settings Replacement trên Windows

### 7.1 Yêu cầu chuẩn
- File cấu hình: `%LOCALAPPDATA%\DropCove\settings.json`.
- Ghi cập nhật cài đặt: Bắt buộc thay thế tập tin nguyên tử (atomically replaced through a temporary file) để chống hỏng hóc dữ liệu khi mất điện hoặc crash đột ngột.
- Nếu file cấu hình thiếu, hỏng cú pháp (malformed) hoặc không đọc được: Tự động khôi phục cấu hình mặc định (fallback to defaults) mà không làm crash tiến trình.

### 7.2 Cơ chế File I/O Nguyên tử trên Windows
Trên Windows NTFS / ReFS:
1. **Hạn chế của `std::fs::rename` thuần túy:**
   - Trong Rust standard library trên Windows, `std::fs::rename(from, to)` gọi xuống `MoveFileExW(from, to, MOVEFILE_REPLACE_EXISTING)`.
   - Nếu tập tin đích `settings.json` đang được một tiến trình quét virus, indexer hoặc text editor mở mà không có chia sẻ `FILE_SHARE_DELETE`, `MoveFileExW` sẽ thất bại với `ERROR_ACCESS_DENIED` hoặc `ERROR_SHARING_VIOLATION`.
2. **Win32 Native API: `ReplaceFileW`:**
   ```cpp
   BOOL ReplaceFileW(
     [in]           LPCWSTR lpReplacedFileName,
     [in]           LPCWSTR lpReplacementFileName,
     [in, optional] LPCWSTR lpBackupFileName,
     [in]           DWORD   dwReplaceFlags,
                    LPVOID  lpExclude,
                    LPVOID  lpReserved
   );
   ```
   - `ReplaceFileW` được thiết kế đặc thù cho việc ghi đè nguyên tử tập tin cấu hình trên Windows:
     - Giữ nguyên File ID, ACLs, creation time và metadata của tập tin gốc.
     - Hỗ trợ tạo backup dự phòng nguyên tử (`lpBackupFileName`).
     - Đảm bảo trạng thái toàn vẹn all-or-nothing.
   - Cần lưu ý: Cả file tạm và file đích phải nằm trên **cùng một volume** (cùng thư mục `%LOCALAPPDATA%\DropCove`).
3. **Quy trình ghi an toàn (Safe Atomic Write Flow):**
   1. Ghi dữ liệu JSON vào `settings.json.tmp` nằm cùng thư mục với `settings.json`.
   2. Gọi `file.sync_all()` (tương đương `FlushFileBuffers`) để đảm bảo dữ liệu đã xả từ OS cache xuống đĩa cứng vật lý.
   3. Nếu `settings.json` chưa tồn tại: Dùng `std::fs::rename(tmp_path, target_path)`.
   4. Nếu `settings.json` đã tồn tại: Gọi `ReplaceFileW` (hoặc `tempfile::NamedTempFile::persist`).
   5. Xóa file tạm nếu còn sót lại khi có lỗi.

### 7.3 Primary Sources
- Microsoft Learn: `ReplaceFileW` function - <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew>
- Microsoft Learn: `MoveFileExW` function - <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw>
- Microsoft Learn: `FlushFileBuffers` function - <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers>

---

## 8. Implications for 0.0.4 tickets

Phần này tóm tắt các tác động kiến trúc và kỹ thuật cụ thể đối với việc phân rã ticket của DropCove 0.0.4:

1. **Ticket Single Instance & Activation:**
   - Sử dụng `tauri-plugin-single-instance`. Cấu hình callback chuyển tiếp các tham số dòng lệnh vào `AppHandle`.
   - Plugin xử lý sẵn `AllowSetForegroundWindow` trước khi exit tiến trình thứ hai, cho phép instance chính gọi `set_focus()` / `show()` mượt mà mà không bị Windows chặn.
2. **Ticket Tray & Menu:**
   - Kích hoạt tính năng menu trực tiếp qua `tauri::tray::TrayIconBuilder` và `tauri::menu::MenuBuilder` sẵn có trong Tauri v2.
   - Bắt sự kiện click chuột trái/double-click để hiển thị shelf tại cursor monitor; click phải mở context menu chuẩn gồm `Settings` và `Exit DropCove`.
3. **Ticket Global Hotkey:**
   - Dùng `tauri-plugin-global-shortcut` (hoặc module Win32 `RegisterHotKey` nội bộ).
   - Bắt buộc kiểm tra cờ `MOD_NOREPEAT` (đã được bật mặc định trong crate `global-hotkey`).
   - Cung cấp cơ chế graceful error handling: Khi trả về `AlreadyRegistered`, phát tín hiệu cảnh báo ra notification area và chuyển sang chế độ chờ thay đổi hotkey từ Settings.
4. **Ticket Autostart Registration:**
   - Tích hợp `tauri-plugin-autostart` với cờ cấu hình `arg("--autostart")`.
   - Đảm bảo đường dẫn thực thi luôn được escape và bọc trong dấu ngoặc kép trên `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
   - Phân nhánh khởi động trong `lib.rs`: Kiểm tra `std::env::args()` xem có `--autostart` không; nếu có thì chuyển thẳng sang chế độ resident ẩn (không show shelf).
5. **Ticket Cursor-Monitor Placement & Topmost Repair:**
   - Tận dụng `windows` crate (đã có trong `Cargo.toml`) để gọi `GetCursorPos`, `MonitorFromPoint`, `GetMonitorInfoW` tính toán chính xác bounding box trên work area.
   - Cài đặt hook `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` để kích hoạt `SetWindowPos(HWND_TOPMOST, SWP_NOACTIVATE)` nhằm bảo vệ z-order của Drop Shelf trước các cửa sổ khác mà không giật focus.
6. **Ticket Settings Storage & Atomic Persistence:**
   - Tạo module lưu trữ `settings.json` trong thư mục cấu hình `%LOCALAPPDATA%\DropCove`.
   - Thực hiện quy trình ghi file tạm kèm `flush` / `sync_all` và hoán đổi nguyên tử bằng `ReplaceFileW` hoặc `std::fs::rename`.
   - Triển khai `serde` deserialize với fallback về default struct khi file trống hoặc bị lỗi cú pháp.
