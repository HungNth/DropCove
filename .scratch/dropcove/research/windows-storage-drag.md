# Nghiên cứu Seam Kỹ thuật Windows Storage Drag & Persistence (DropCove Ticket 01)

Tài liệu này xác định API tối thiểu, đáng tin cậy cho DropCove `0.0.2` (Ticket 01: Single-item Drop Shelf round trip) trên nền tảng Windows 11 x64 MSVC (`cargo-xwin`), dựa hoàn toàn trên primary sources.

---

## 1. Tauri v2 File/Folder Drag-In (Inbound Storage Drag)

### 1.1 Khả năng tiếp nhận và cơ chế hoạt động
Tauri v2 hỗ trợ sẵn tiếp nhận drag-in tập tin/thư mục từ Windows Explorer hoặc ứng dụng desktop khác vào Webview/Window thông qua cơ chế tích hợp sẵn của Tauri runtime (`tauri-runtime-wry` / `wry`).
Trên Windows, Tauri/Wry tắt tính năng external drop mặc định của WebView2 (`SetAllowExternalDrop(false)`) và đăng ký OLE `IDropTarget` (`RegisterDragDrop`) trên HWND của webview để bắt các drop payload định dạng `CF_HDROP`.

- **Event Payload Type:** `DragDropEvent`
  - `DragDropEvent::Enter { paths: Vec<PathBuf>, position: PhysicalPosition<f64> }`
  - `DragDropEvent::Over { position: PhysicalPosition<f64> }`
  - `DragDropEvent::Drop { paths: Vec<PathBuf>, position: PhysicalPosition<f64> }`
  - `DragDropEvent::Leave`
- **Dữ liệu path:** Trả về danh sách đường dẫn tuyệt đối dạng `Vec<PathBuf>` (ở Rust) hoặc `string[]` (ở JS/TS frontend). Cả file lẫn folder đều được OS cung cấp đường dẫn đầy đủ trong `paths`.

### 1.2 Các điểm bắt sự kiện (Event Hooks)
Có 2 điểm tiếp nhận tùy thuộc vào tầng xử lý:

1. **Rust Backend Handler (Khuyến nghị cho Core State / Windows Drag Adapter):**
   - Đăng ký qua `tauri::Builder::on_window_event` hoặc `tauri::Builder::on_webview_event`:
     ```rust
     tauri::Builder::default()
         .on_window_event(|window, event| {
             if let tauri::WindowEvent::DragDrop(drag_drop_event) = event {
                 match drag_drop_event {
                     tauri::DragDropEvent::Drop { paths, position } => {
                         // Xử lý nạp items vào Core Shelf State trực tiếp tại backend
                     }
                     _ => {}
                 }
             }
         })
     ```
   - **Ưu điểm:** Đường dẫn `PathBuf` đi trực tiếp vào Rust domain logic mà không phụ thuộc IPC frontend; không bị chặn bởi permission hay CSP; tuân thủ mô hình "State owns intake".

2. **Frontend JS/TS Listener:**
   - Dùng API `@tauri-apps/api/webview` hoặc `@tauri-apps/api/webviewWindow`:
     - Method helper: `getCurrentWebview().onDragDropEvent((event) => ...)`
     - Hoặc listen event tauri nội bộ: `listen("tauri://drag-drop", (event) => ...)` (các event tương ứng: `tauri://drag-enter`, `tauri://drag-over`, `tauri://drag-drop`, `tauri://drag-leave`).
     - Payload nhận được có dạng `{ type: 'drop', paths: string[], position: { x: number, y: number } }`.

### 1.3 Cấu hình và Capability cần thiết
- **Cấu hình `tauri.conf.json`:**
  - Thuộc tính `app.windows[].dragDropEnabled`: mặc định là `true`.
  - **Lưu ý quan trọng:** Không được đặt `dragDropEnabled: false`. Nếu đặt `false`, Tauri sẽ không gắn `IDropTarget` và nhường lại cho HTML5 Drag & Drop thông thường của trình duyệt (HTML5 drag & drop trong browser engine không đọc được absolute filesystem paths vì lý do sandbox).
- **Quyền hạn / Capabilities (`capabilities/*.json`):**
  - Nếu xử lý drag-in hoàn toàn tại **Rust backend** (`on_window_event` / `on_webview_event`): **Không yêu cầu bất kỳ capability hay IPC permission nào**.
  - Nếu nhận sự kiện tại **Frontend JS** (`onDragDropEvent` hoặc `listen`): Cần permission `core:event:default` (cho phép `allow-listen`, vốn đã nằm sẵn trong `core:default` tại `src-tauri/capabilities/default.json`).

### 1.4 Primary Sources
- Tauri v2 `DragDropEvent` docs: <https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html>
- Tauri v2 `WindowEvent` docs: <https://docs.rs/tauri/latest/tauri/enum.WindowEvent.html>
- Tauri v2 `Builder::on_window_event` docs: <https://docs.rs/tauri/latest/tauri/struct.Builder.html>
- Tauri v2 Webview JS API: <https://v2.tauri.app/reference/javascript/api/namespacewebview/>
- Wry Windows `drag_drop.rs` source: <https://docs.rs/crate/wry/latest/source/src/webview2/drag_drop.rs>

---

## 2. Outbound Storage Drag trong Tauri v2

### 2.1 Tauri v2 Core có API outbound storage drag không?
**Câu trả lời: KHÔNG.**
- Trong Tauri core v2 (`tauri` crate 2.12), chỉ có API kéo di chuyển cửa sổ:
  - `Window::start_dragging(&self)`: bắt đầu thao tác kéo cửa sổ bằng chuột (window dragging / move window theo con trỏ chuột, liên quan HTML attribute `data-tauri-drag-region`).
  - `Window::start_resize_dragging(&self, direction)`: kéo để resize kích thước cửa sổ.
- Tauri v2 core **hoàn toàn không có API** để khởi tạo OLE data transfer hoặc kéo file/folder ra ngoài màn hình/ứng dụng khác (outbound storage drag-out).

### 2.2 Các plugin chính thức hoặc cộng đồng (`tauri-plugin-drag` / `drag`)
- Plugin `tauri-plugin-drag` (phiên bản `2.1.1` của CrabNebula) gọi xuống crate `drag` (phiên bản `2.1.1`).
- **Phân tích crate `drag` trên Windows (`DoDragDrop`):**
  1. Nhận danh sách đường dẫn `DragItem::Files(Vec<PathBuf>)`.
  2. Tạo `IDataObject` thông qua Shell API: `SHCreateShellItemArrayFromIDLists` -> `shell_item_array.BindToHandler(None, &BHID_DataObject)`.
  3. Khởi tạo `IDropSource` (thực thi `QueryContinueDrag` và `GiveFeedback`).
  4. Gọi Windows OLE API:
     ```rust
     let drop_result = DoDragDrop(&data_object, &drop_source, effect, &mut out_dropeffect);
     ```
  5. Đánh giá kết quả trả về trong `drag`:
     ```rust
     if drop_result == DRAGDROP_S_DROP {
         on_drop_callback(DragResult::Dropped, CursorPosition { x: pt.x, y: pt.y });
     } else {
         // DRAGDROP_S_CANCEL
         on_drop_callback(DragResult::Cancel, CursorPosition { x: pt.x, y: pt.y });
     }
     ```

### 2.3 Tại sao `tauri-plugin-drag` / `drag` KHÔNG đáp ứng contract DropCove?
Theo đặc tả của DropCove (Architecture §8 và Issue 01 Acceptance Criteria 3, 4):
> *"Offer copy-only item drag-out. Remove the Temporary Item only after the destination returns accepted Copy; retain it after rejection, cancellation, resolution failure, or pre-acceptance failure."*

Crate `drag` có 2 thiếu sót chí mạng đối với contract này:
1. **Bỏ qua giá trị `pdwEffect` (out_dropeffect):**
   Trong Windows API `DoDragDrop`:
   ```cpp
   HRESULT DoDragDrop(
     [in]  LPDATAOBJECT pDataObj,
     [in]  LPDROPSOURCE pDropSource,
     [in]  DWORD        dwOKEffects,
     [out] LPDWORD      pdwEffect
   );
   ```
   Khi đích đến thả chuột, hàm trả về `DRAGDROP_S_DROP`. Tuy nhiên, kết quả hành động thực tế của đích đến được ghi vào `*pdwEffect`:
   - Nếu đích đến chấp nhận Copy: `*pdwEffect & DROPEFFECT_COPY != 0`.
   - Nếu đích đến từ chối hoặc chọn None: `*pdwEffect == DROPEFFECT_NONE` (ví dụ kéo vào vùng không hỗ trợ drop nhưng target xử lý DROP trả về S_DROP, hoặc target không nhận copy).
   - Crate `drag` **hoàn toàn bỏ qua `out_dropeffect`**, chỉ kiểm tra `drop_result == DRAGDROP_S_DROP` rồi báo chung là `DragResult::Dropped`.
2. **Enum `DragResult` chỉ có 2 biến thể: `Dropped` và `Cancel`:**
   Không phân biệt được giữa `Copy` (thành công chấp nhận copy) và `Rejected` / `DROPEFFECT_NONE`. Nếu dùng `drag`, mọi thao tác kết thúc bằng drop sẽ bị ngộ nhận là copy thành công, vi phạm nghiêm trọng invariant *"Retaining an extra reference is preferred over incorrectly removing one"*.
3. **Cần `dunce::canonicalize` tự ý thay đổi đường dẫn:**
   Trong mã nguồn `drag::start_drag`, nó tự động chạy `dunce::canonicalize` trên các file path trước khi đưa vào OLE array. Trong khi đó đặc tả DropCove §7.1 quy định: *"Paths are not canonicalized and separators are not rewritten."*

### 2.4 Primary Sources
- Windows `DoDragDrop` API: <https://learn.microsoft.com/en-us/windows/win32/api/ole2/nf-ole2-dodragdrop>
- Windows `DROPEFFECT` constants: <https://learn.microsoft.com/en-us/windows/win32/com/dropeffect-constants>
- Crate `drag` v2.1.1 source code: <https://docs.rs/crate/drag/2.1.1/source/src/platform_impl/windows/mod.rs>

---

## 3. Windows Storage Drag-Out Adapter (OLE / Win32)

Do Tauri v2 và các plugin bên thứ 3 không đáp ứng contract trả về `Copy` vs `Rejected` vs `Canceled`, DropCove cần một **Windows Storage Drag Adapter riêng**.

### 3.1 Windows API tối thiểu
Cơ chế chuẩn trên Windows từ Windows 2000 đến Windows 11 cho clipboard và drag-out file references là OLE Drag and Drop với định dạng clipboard `CF_HDROP`:
1. **Khởi tạo OLE/COM:** `OleInitialize()` (hoặc `CoInitializeEx`).
2. **Định dạng dữ liệu `CF_HDROP` (`DROPFILES`):**
   - Cấu trúc `DROPFILES`:
     ```cpp
     typedef struct _DROPFILES {
       DWORD pFiles; // offset tính bằng bytes tới chuỗi ký tự file list
       POINT pt;     // drop point
       BOOL  fNC;    // non-client flag
       BOOL  fWide;  // TRUE nếu chuỗi UTF-16 wchar_t
     } DROPFILES;
     ```
   - Sau header `DROPFILES` là danh sách các đường dẫn file dưới dạng double-null-terminated wide string: `path1\0path2\0\0`.
   - Cấp phát qua `GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size)` và khóa bằng `GlobalLock`.
3. **COM Interface `IDataObject`:**
   - Cung cấp `GetData` trả về `STGMEDIUM` với `TYMED_HGLOBAL` chứa handle của `DROPFILES` khi được hỏi format `CF_HDROP`.
   - Hoặc dùng Shell helper `SHCreateDataObject` / `SHCreateShellItemArrayFromIDLists` (như cách `drag` crate làm nhưng đơn giản hơn là tự tạo `IDataObject` chứa `CF_HDROP`).
4. **COM Interface `IDropSource`:**
   - Cài đặt 2 phương thức:
     - `QueryContinueDrag(fEscapePressed, grfKeyState)`:
       - Nếu phím ESC được nhấn (`fEscapePressed != 0`) -> trả về `DRAGDROP_S_CANCEL`.
       - Nếu nút chuột trái nhả ra (`(grfKeyState & MK_LBUTTON) == 0`) -> trả về `DRAGDROP_S_DROP`.
       - Ngược lại -> trả về `S_OK`.
     - `GiveFeedback(dwEffect)`:
       - Trả về `DRAGDROP_S_USEDEFAULTCURSORS` (hệ thống tự hiển thị con trỏ chuột Copy / No-Drop).
5. **Thực thi `DoDragDrop`:**
   ```rust
   let dw_ok_effects = DROPEFFECT_COPY; // CHỈ cho phép Copy (Copy-only)
   let mut dw_effect = DROPEFFECT_NONE;

   let hr = DoDragDrop(
       &data_object,
       &drop_source,
       dw_ok_effects,
       &mut dw_effect,
   );
   ```
6. **Xác định kết quả (Outcome Mapping):**
   - Nếu `hr == DRAGDROP_S_DROP`:
     - Nếu `(dw_effect & DROPEFFECT_COPY) != 0`: **Kết quả = `DragOutcome::Copy`** (Destination chấp nhận copy).
     - Nếu `dw_effect == DROPEFFECT_NONE`: **Kết quả = `DragOutcome::Rejected`** (Destination từ chối hoặc thả vào vùng không nhận).
   - Nếu `hr == DRAGDROP_S_CANCEL`: **Kết quả = `DragOutcome::Canceled`**.
   - Nếu `hr` thất bại khác: **Kết quả = `DragOutcome::Failed`**.

### 3.2 Rust Crate phù hợp nhất
- **Lựa chọn:** Crate chính chủ của Microsoft: `windows` (hoặc `windows-sys`).
  - Trong dự án hiện tại, dự án dùng toolchain `cargo-xwin` cho target `x86_64-pc-windows-msvc`.
  - Crate `windows` (với các features: `Win32_System_Ole`, `Win32_System_Com`, `Win32_System_Memory`, `Win32_UI_Shell`, `Win32_Foundation`) cung cấp đầy đủ bindings, macro `implement` cho `IDataObject` và `IDropSource`.
  - Không cần thêm bất kỳ dependency C-runtime hay DLL bên ngoài nào (các DLL OLE32, Shell32, Kernel32 có sẵn trên tất cả các phiên bản Windows).

### 3.3 Giới hạn tích hợp WebView / Tauri & Threading Model
1. **Threading Model cho `DoDragDrop` (Đã đính chính từ kết quả smoke thực tế):**
   - Ban đầu có thể giả định dispatch `DoDragDrop` sang background worker thread để tránh block UI. Tuy nhiên, **thực nghiệm smoke test trực tiếp trên Tauri đã chứng minh: background worker dispatch bị treo/busy vĩnh viễn**, trong khi gọi `DoDragDrop` trên **Tauri owning-window thread (main UI thread)** hoạt động chuẩn xác và mượt mà.
   - **Nguyên nhân kỹ thuật:** `DoDragDrop` yêu cầu message queue liên kết với HWND sở hữu con trỏ và mouse capture. Khi người dùng click chuột trong WebView2 để bắt đầu kéo, sự kiện chuột gắn với UI thread. Nếu gọi `DoDragDrop` trên worker thread, thread đó không sở hữu message pump của window và không nhận được các mouse messages cần thiết để điều khiển quá trình drag.
   - Ngược lại, khi thực thi trên main UI thread (thông qua `app.run_on_main_thread(...)` hoặc direct invocation từ main thread), `DoDragDrop` tự chạy một Win32 modal message pump nội bộ, tiếp tục dispatch các message hệ thống cần thiết trong khi theo dõi chuyển động chuột cho đến khi nhả chuột hoặc bấm ESC.
2. **Khởi động Drag từ Webview:**
   - Trong giao diện HTML/Vue, khi người dùng nhấn giữ chuột và kéo một item card (`mousedown` + `mousemove` vượt ngưỡng drag threshold hoặc HTML5 drag event), frontend gửi Tauri command `start_item_drag(item_id)` về backend.
   - Backend chuẩn bị danh sách path, gọi Windows Storage Drag Adapter trên main thread, thực thi OLE `DoDragDrop`, và nhận kết quả enum (`Copy`, `Rejected`, `Canceled`) để hoàn tất mutation trong Core Shelf State.
### 3.4 Primary Sources
- Microsoft Docs - `DoDragDrop`: <https://learn.microsoft.com/en-us/windows/win32/api/ole2/nf-ole2-dodragdrop>
- Microsoft Docs - `DROPFILES`: <https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ns-shlobj_core-dropfiles>
- Microsoft Docs - `IDropSource`: <https://learn.microsoft.com/en-us/windows/win32/api/oleidl/nn-oleidl-idropsource>
- Microsoft Docs - Transferring Shell Objects with Drag-and-Drop and the Clipboard: <https://learn.microsoft.com/en-us/windows/win32/shell/datascrape>

---

## 4. Lựa chọn SQLite Đồng Bộ Cho State Serialized

### 4.1 Crate đề xuất: `rusqlite`
- **Phiên bản mới nhất:** `rusqlite = "0.40.2"` (hoặc `0.33+`).
- **Feature cần bật:** `features = ["bundled"]`.

### 4.2 Đánh giá các tiêu chí theo yêu cầu kiến trúc DropCove
1. **Mô hình đồng bộ & Serialized Access:**
   - DropCove quy định một State Owner duy nhất (`Core Shelf State`), bảo đảm serialize toàn bộ các thay đổi vào database.
   - `rusqlite::Connection` là kiểu kết nối đồng bộ, đơn luồng trực tiếp, chi phí cực nhẹ (zero overhead so với sqlx hay orm).
   - Có thể bọc `Connection` trong một Mutex (`Arc<Mutex<Connection>>`) hoặc sở hữu trực tiếp bởi Core State thread/actor.
2. **Write-Ahead Logging (WAL):**
   - SQLite hỗ trợ chế độ WAL thông qua câu lệnh PRAGMA chuẩn:
     ```rust
     conn.pragma_update(None, "journal_mode", "WAL")?;
     ```
   - Thực thi thành công ngay sau khi mở kết nối. Khi ở chế độ WAL, các thao tác đọc và ghi không khoá lẫn nhau, đảm bảo hiệu năng và tính bền vững khi restart.
3. **Foreign Keys:**
   - Mặc định SQLite tắt foreign key constraint. Bật bằng câu lệnh PRAGMA:
     ```rust
     conn.pragma_update(None, "foreign_keys", "ON")?;
     ```
   - Giúp đảm bảo quan hệ cha-con giữa `shelf_batches` và `shelf_items` được thực thi nghiêm ngặt tại tầng database.
4. **Integrity Check on Open:**
   - Kiểm tra toàn vẹn dữ liệu khi khởi động ứng dụng:
     ```rust
     let mut stmt = conn.prepare("PRAGMA quick_check")?; // hoặc "PRAGMA integrity_check"
     let result: String = stmt.query_row([], |row| row.get(0))?;
     if result != "ok" {
         return Err(ShelfError::DatabaseCorrupt(result));
     }
     ```
   - Đáp ứng trực tiếp yêu cầu: *"Refuse unsafe state and report failure... without modifying the database"*.
5. **Hỗ trợ Transaction:**
   - `rusqlite` cung cấp Transaction RAII an toàn:
     ```rust
     let tx = conn.transaction()?;
     // thực hiện batch insert hoặc update
     tx.commit()?;
     ```
   - Nếu xảy ra lỗi hoặc drop trước khi commit, toàn bộ thay đổi tự động rollback, bảo đảm tiêu chuẩn: *"A forced persistence failure produces no visible partial add, removal, or drag completion"*.

### 4.3 Windows Cross-Build Implications (`cargo-xwin` & MSVC)
- **Vấn đề khi build SQLite trên Windows:**
  - Nếu không cấu hình, SQLite FFI thường tìm kiếm thư viện SQLite có sẵn của hệ thống qua `pkg-config`, vốn không tồn tại trên môi trường Windows thông thường hoặc môi trường cross-compile.
  - Khi bật `features = ["bundled"]`, crate `libsqlite3-sys` (phụ thuộc của `rusqlite`) sẽ tự động biên dịch mã nguồn C của SQLite (amalgamation `sqlite3.c`) đi kèm bằng `cc` crate.
  - `cargo-xwin` đóng vai trò wrapper cho clang/lld mô phỏng MSVC (`x86_64-pc-windows-msvc`). Crate `cc` hoạt động tương thích hoàn toàn với `cargo-xwin` mà không cần cài đặt Visual Studio C++ build tools.
  - Kết quả tạo ra binary tĩnh độc lập, không phụ thuộc vào `sqlite3.dll` ngoài máy người dùng.

### 4.4 Primary Sources
- Rusqlite Documentation: <https://docs.rs/rusqlite/latest/rusqlite/>
- SQLite PRAGMA journal_mode: <https://www.sqlite.org/pragma.html#pragma_journal_mode>
- SQLite PRAGMA integrity_check: <https://www.sqlite.org/pragma.html#pragma_integrity_check>
- Rusqlite GitHub bundled feature notes: <https://github.com/rusqlite/rusqlite#bundled>

---

## 5. Đề Xuất Seam Tối Thiểu Cho Ticket 01

Để hoàn thành Ticket 01 mà không phát sinh abstraction dư thừa (tuân thủ nguyên tắc Ponytail), seam kiến trúc được phân tách thành các thành phần tối thiểu sau:

```
[Windows OS Explorer / Desktop]
     │ (Drag-in: CF_HDROP)
     ▼
[Tauri WindowEvent::DragDrop] ──── (1. Paths) ────► [Core Shelf State]
                                                          │
     ┌────────────────────────────────────────────────────┤
     │                                                    ▼
     ▼                                        [Shelf Persistence Adapter]
[Presentation (Webview/Vue)]                      (rusqlite: shelf.db)
  - Render Card                                   - WAL, Foreign Keys
  - User drags item                               - integrity_check on open
     │
     ▼ (Invoke start_drag)
[Windows Storage Drag Adapter]
  (windows crate: DoDragDrop)
  - Copy-only (DROPEFFECT_COPY)
  - Maps outcome: Copy | Rejected | Canceled
     │
     └───────────── (2. DragOutcome) ───────────► [Core Shelf State]
                                                   - If Copy: remove item & batch durably
                                                   - If Rejected/Canceled: retain item
```

### 5.1 Các Module cụ thể

1. **`ShelfStore` (Shelf Persistence Adapter):**
   - Đặt tại `src-tauri/src/store/mod.rs` (hoặc `db.rs`).
   - Sử dụng `rusqlite = { version = "0.40", features = ["bundled"] }`.
   - Kết nối `%LOCALAPPDATA%\DropCove\shelf.db`.
   - Khởi tạo: kiểm tra `PRAGMA integrity_check`, bật `PRAGMA journal_mode = WAL`, `PRAGMA foreign_keys = ON`, tạo bảng `shelf_batches` và `shelf_items` theo schema ARCHITECTURE.md §10.1.
   - Cung cấp các thao tác transactional atomic: `load_all()`, `insert_batch()`, `remove_item()`, `remove_batch()`.

2. **`WindowsDragAdapter` (Windows Storage Drag Adapter):**
   - Đặt tại `src-tauri/src/drag/mod.rs` (hoặc `windows_drag.rs`).
   - Sử dụng crate `windows` (chỉ bật các module cần thiết: `Win32_System_Ole`, `Win32_System_Com`, `Win32_System_Memory`, `Win32_UI_Shell`, `Win32_Foundation`).
   - Cung cấp 1 hàm duy nhất:
     ```rust
     pub fn perform_storage_drag(paths: &[PathBuf]) -> Result<DragOutcome, DragError>;
     ```
   - Định nghĩa enum `DragOutcome`:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub enum DragOutcome {
         Copy,      // Destination returned DROPEFFECT_COPY
         Rejected,  // Destination returned DROPEFFECT_NONE or drop unhandled
         Canceled,  // DRAGDROP_S_CANCEL (user pressed ESC or canceled)
     }
     ```

3. **`CoreShelfState` (State Owner):**
   - Đặt tại `src-tauri/src/state/mod.rs`.
   - Lưu trữ state trong bộ nhớ: danh sách batches và items.
   - Sở hữu một instance của `ShelfStore`.
   - Các mutation methods:
     - `intake_paths(paths: Vec<PathBuf>) -> Result<Option<BatchId>, StateError>`: lọc empty path, deduplicate trong payload, kiểm tra duplicate batch theo quy tắc ordinal case-insensitive, ghi vào DB và update memory.
     - `prepare_item_drag(item_id: &str) -> Result<PathBuf, StateError>`.
     - `complete_item_drag(item_id: &str, outcome: DragOutcome) -> Result<bool, StateError>`: nếu `outcome == DragOutcome::Copy`, xóa item khỏi DB và bộ nhớ; nếu batch trống, xóa batch. Ngược lại giữ nguyên.

4. **Kênh tiếp nhận & Presentation Hook:**
   - **Drag-in:** Hook trực tiếp qua `tauri::Builder::on_window_event`. Khi nhận `WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. })`, gọi `state.intake_paths(paths)`. Sau đó emit event `shelf://updated` để frontend render lại.
   - **Drag-out:** Frontend bắt thao tác kéo thẻ (card `mousedown` hoặc drag hook) và gọi Tauri command `start_item_drag(item_id)`. Backend chạy `perform_storage_drag`, nhận `DragOutcome`, gọi `complete_item_drag(item_id, outcome)` và trả kết quả về cho frontend.

---

## 6. Custom Inbound IDropTarget cho Ticket 02 (Mixed Storage Items & Virtual Shell Entries)

Ticket 02 mở rộng yêu cầu tiếp nhận payload kéo vào: không chỉ đọc các đường dẫn filesystem thuần túy từ `CF_HDROP`, mà phải hỗ trợ **mixed storage items (chấp nhận các item hợp lệ và đếm chính xác số lượng item bị bỏ qua - `skipped_count`)**, đồng thời xử lý các virtual shell entries (như Recycle Bin, This PC, Control Panel items...).

### 6.1 Giới hạn của Wry/Tauri v2 Inbound Target
Kiểm tra mã nguồn `wry-0.57.0/src/webview2/drag_drop.rs` cho thấy:
1. Wry chỉ tìm định dạng `CF_HDROP` (`drop_format.cfFormat = CF_HDROP.0`) qua `data_obj.GetData(&drop_format)` và phân giải tên file qua `DragQueryFileW`.
2. Nếu người dùng kéo các virtual shell items hoặc mixed items, Wry không phân tích `IShellItemArray` mà chỉ đọc `HDROP`. Bất kỳ item nào không nằm trong `HDROP` sẽ bị bỏ qua hoàn toàn mà không thể đếm được.
3. Wry không cung cấp bất kỳ API hay payload nào để chuyển `skipped_count` về cho ứng dụng.
4. Do đó, để đáp ứng Ticket 02, DropCove **bắt buộc phải cài đặt một custom `IDropTarget`** thay thế target mặc định của Wry.

### 6.2 Cài đặt Custom `IDropTarget` với `SHCreateShellItemArrayFromDataObject`

#### Cơ chế phân giải:
Thay vì chỉ dựa vào `CF_HDROP`, Windows Shell cung cấp API chuẩn:
```cpp
SHSTDAPI SHCreateShellItemArrayFromDataObject(
  [in]  IDataObject *pdo,
  [in]  REFIID      riid, // IID_IShellItemArray
  [out] void        **ppv // IShellItemArray
);
```
API này đọc được cả `CF_HDROP`, `CFSTR_SHELLIDLIST` (HIDA) và các định dạng shell nội bộ khác để tái tạo mảng `IShellItemArray`.

#### Thuật toán phân loại & giữ nguyên thứ tự nguồn (Source-Order Preservation):
```rust
use std::path::PathBuf;
use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::{CoTaskMemFree, IDataObject};
use windows::Win32::UI::Shell::{
    SHCreateShellItemArrayFromDataObject, IShellItem, IShellItemArray, SIGDN_FILESYSPATH,
};

pub fn extract_storage_items(
    data_obj: &IDataObject,
) -> windows::core::Result<(Vec<PathBuf>, usize)> {
    let shell_item_array: IShellItemArray = unsafe {
        SHCreateShellItemArrayFromDataObject(data_obj)?
    };

    let count = unsafe { shell_item_array.GetCount()? };
    let mut accepted_paths = Vec::new();
    let mut skipped_count = 0usize;

    // Duyệt theo đúng thứ tự nguồn từ 0 đến count - 1
    for i in 0..count {
        let item: IShellItem = unsafe { shell_item_array.GetItemAt(i)? };
        // SIGDN_FILESYSPATH chỉ thành công với filesystem items (SFGAO_FILESYSTEM)
        match unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
            Ok(pwstr) => {
                let path_str = unsafe { pwstr.to_string() };
                unsafe { CoTaskMemFree(Some(pwstr.as_ptr() as _)) };
                if let Ok(s) = path_str {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        accepted_paths.push(PathBuf::from(s));
                        continue;
                    }
                }
                skipped_count += 1;
            }
            Err(_) => {
                // Virtual shell item (không có đường dẫn filesystem) -> tính vào skipped
                skipped_count += 1;
            }
        }
    }

    Ok((accepted_paths, skipped_count))
}
```

### 6.3 Copy Allowed-Effect Filtering
Đặc tả DropCove yêu cầu: chỉ chấp nhận thao tác kéo thả nếu nguồn cho phép **Copy** (`DROPEFFECT_COPY`).

Trong cài đặt `IDropTarget_Impl`:
1. **`DragEnter(&self, pDataObj, _grfKeyState, _pt, pdwEffect)`:**
   ```rust
   let allowed_effects = unsafe { *pdwEffect };
   if (allowed_effects.0 & DROPEFFECT_COPY.0) == 0 {
       // Nguồn không cho phép Copy -> từ chối ngay lập tức
       unsafe { *pdwEffect = DROPEFFECT_NONE };
       return Ok(());
   }

   // Kiểm tra xem payload có item nào hợp lệ hay không
   if let Ok((accepted, _skipped)) = extract_storage_items(&pDataObj) {
       if !accepted.is_empty() {
           unsafe { *pdwEffect = DROPEFFECT_COPY };
           return Ok(());
       }
   }

   // Không có item filesystem hợp lệ nào -> từ chối
   unsafe { *pdwEffect = DROPEFFECT_NONE };
   ```
2. **`DragOver(&self, _grfKeyState, _pt, pdwEffect)`:**
   - Trả lại hiệu ứng đã xác định ở `DragEnter` (`DROPEFFECT_COPY` hoặc `DROPEFFECT_NONE`).
3. **`Drop(&self, pDataObj, _grfKeyState, _pt, pdwEffect)`:**
   - Kiểm tra `(allowed_effects.0 & DROPEFFECT_COPY.0) != 0`.
   - Trích xuất `(accepted_paths, skipped_count)`.
   - Nếu `!accepted_paths.is_empty()`:
     - Đẩy vào Core Shelf State: `state.accept_drop(accepted_paths, skipped_count)`.
     - Gán `*pdwEffect = DROPEFFECT_COPY`.
   - Ngược lại:
     - Gán `*pdwEffect = DROPEFFECT_NONE`.

### 6.4 Xác định HWND & Thay thế Wry Target (Tránh Double Registration)

#### 1. Cấu trúc cửa sổ WebView2:
Trong Tauri trên Windows, cửa sổ gồm:
- Top-level HWND (cửa sổ Tauri Win32 chính).
- Các child HWND do WebView2 tạo ra để render nội dung (ví dụ `Chrome_WidgetWin_0`, `Intermediate D3D Window`).
Wry injects `IDropTarget` bằng cách gọi `EnumChildWindows(top_hwnd, ...)` và đăng ký trên mọi child HWND.

#### 2. Lấy HWND:
- Từ `WebviewWindow`:
  ```rust
  let top_hwnd: HWND = window.hwnd()?;
  ```
- Hoặc từ `with_webview`:
  ```rust
  window.with_webview(|platform_webview| {
      let controller = platform_webview.controller();
      let mut parent_hwnd = HWND::default();
      unsafe { controller.get_ParentWindow(&mut parent_hwnd as *mut _ as _) };
  })?;
  ```

#### 3. Thay thế Wry Target mà không gặp lỗi `DRAGDROP_E_ALREADYREGISTERED`:
OLE không cho phép đăng ký 2 `IDropTarget` cùng lúc trên một HWND. Nếu gọi `RegisterDragDrop` khi đã có target, OLE trả về lỗi `DRAGDROP_E_ALREADYREGISTERED` (0x80040101).

Quy trình hoán đổi an toàn:
1. Liệt kê toàn bộ child HWND bằng `EnumChildWindows(top_hwnd, ...)`.
2. Với mỗi child `hwnd`:
   ```rust
   // Thu hồi target cũ của Wry trước
   let _ = unsafe { RevokeDragDrop(hwnd) };
   // Đăng ký custom IDropTarget của DropCove
   let res = unsafe { RegisterDragDrop(hwnd, &custom_drop_target) };
   ```
3. Lưu giữ `custom_drop_target` (giữ COM reference count) trong controller/adapter tồn tại suốt vòng đời của window.

### 6.5 Registration & Drop Lifetime Invariants
1. **`IDataObject` Lifetime:**
   - Theo tài liệu Microsoft Learn: *"The resulting shell item array holds a reference to the source data object. Notably, the data objects passed to IDropTarget methods are no longer valid after the drop operation completes."*
   - **Quy tắc bắt buộc:** Việc gọi `extract_storage_items` (bao gồm `SHCreateShellItemArrayFromDataObject`, duyệt `GetItemAt`, trích xuất `SIGDN_FILESYSPATH` thành các owned `PathBuf`) **phải hoàn tất đồng bộ ngay trong thân hàm `Drop` (hoặc `DragEnter`)**. Không được lưu trữ `IDataObject` hoặc `IShellItemArray` thô để xử lý bất đồng bộ sau khi hàm `Drop` đã return.
2. **`IDropTarget` Registration Lifetime:**
   - Đối tượng cài đặt `IDropTarget` phải được giữ sống (alive) trong struct adapter gắn với window.
   - Khi window bị hủy (`Destroyed` event), gọi `RevokeDragDrop(hwnd)` cho các HWND đã đăng ký để dọn dẹp sạch sẽ tài nguyên COM.

### 6.6 Windows Crate Features Cần Thiết
Trong `src-tauri/Cargo.toml`:
```toml
[dependencies.windows]
version = "0.58" # hoặc bản tương đương đang dùng trong ecosystem
features = [
    "Win32_Foundation",
    "Win32_System_Ole",
    "Win32_System_Com",
    "Win32_System_Memory",
    "Win32_UI_Shell",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
]
```

### 6.7 Primary Sources
- Microsoft Docs - `SHCreateShellItemArrayFromDataObject`: <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateshellitemarrayfromdataobject>
- Microsoft Docs - `IShellItemArray`: <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellitemarray>
- Microsoft Docs - `IShellItem::GetDisplayName`: <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem-getdisplayname>
- Microsoft Docs - `SIGDN` enumeration: <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-sigdn>
- Microsoft Docs - `RegisterDragDrop` & `RevokeDragDrop`: <https://learn.microsoft.com/en-us/windows/win32/api/ole2/nf-ole2-registerdragdrop>
- Microsoft Docs - `ICoreWebView2Controller`: <https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controller>

