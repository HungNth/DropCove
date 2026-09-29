using System.Runtime.InteropServices;

namespace DropCove.Native;

/// <summary>Contains an in-memory 32-bit Windows Shell icon image.</summary>
/// <param name="Width">The image width in pixels.</param>
/// <param name="Height">The image height in pixels.</param>
/// <param name="BgraPixels">The top-down 32-bit BGRA pixels.</param>
public sealed record ShellIconImage(int Width, int Height, byte[] BgraPixels);

/// <summary>Loads native file and folder icons through the Windows Shell.</summary>
public static class ShellIconLoader
{
    private const uint FileAttributeDirectory = 0x00000010;
    private const uint FileAttributeNormal = 0x00000080;
    private const uint ShgfiIcon = 0x000000100;
    private const uint ShgfiSmallIcon = 0x000000001;
    private const uint ShgfiUseFileAttributes = 0x000000010;
    private const uint BiRgb = 0;
    private const uint DibRgbColors = 0;
    private const uint DiNormal = 0x0003;

    /// <summary>Loads one native icon without blocking the caller's thread.</summary>
    /// <param name="path">The filesystem path used to select the Shell icon.</param>
    /// <param name="isFolder">Whether the path represents a folder.</param>
    /// <param name="cancellationToken">A token that discards the result when no longer visible.</param>
    /// <returns>The icon pixels, or <see langword="null" /> when Shell could not provide an icon.</returns>
    /// <exception cref="ArgumentNullException">The path is <see langword="null" />.</exception>
    /// <exception cref="ArgumentException">The path is empty or whitespace.</exception>
    /// <exception cref="OperationCanceledException">The request is canceled before the icon is loaded.</exception>
    public static async Task<ShellIconImage?> LoadAsync(
        string path,
        bool isFolder,
        CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        cancellationToken.ThrowIfCancellationRequested();

        var image = await Task.Run(
            () => Load(path, isFolder),
            cancellationToken).ConfigureAwait(false);
        cancellationToken.ThrowIfCancellationRequested();
        return image;
    }

    private static ShellIconImage? Load(string path, bool isFolder)
    {
        var attributes = isFolder ? FileAttributeDirectory : FileAttributeNormal;
        var flags = ShgfiIcon | ShgfiSmallIcon;
        if (!File.Exists(path) && !Directory.Exists(path))
        {
            flags |= ShgfiUseFileAttributes;
        }

        if (SHGetFileInfo(
                path,
                attributes,
                out var shellFileInfo,
                (uint)Marshal.SizeOf<SHFILEINFO>(),
                flags) == 0 || shellFileInfo.Icon == 0)
        {
            return null;
        }

        try
        {
            return RenderIcon(shellFileInfo.Icon);
        }
        finally
        {
            DestroyIcon(shellFileInfo.Icon);
        }
    }

    private static ShellIconImage? RenderIcon(nint icon)
    {
        const int size = 32;
        var screenDc = GetDC(0);
        if (screenDc == 0)
        {
            return null;
        }

        nint memoryDc = 0;
        nint dib = 0;
        nint previous = 0;
        try
        {
            memoryDc = CreateCompatibleDC(screenDc);
            if (memoryDc == 0)
            {
                return null;
            }

            var bitmapInfo = new BITMAPINFO
            {
                Header = new BITMAPINFOHEADER
                {
                    Size = (uint)Marshal.SizeOf<BITMAPINFOHEADER>(),
                    Width = size,
                    Height = -size,
                    Planes = 1,
                    BitCount = 32,
                    Compression = BiRgb,
                },
            };

            dib = CreateDIBSection(screenDc, ref bitmapInfo, DibRgbColors, out var bits, 0, 0);
            if (dib == 0 || bits == 0)
            {
                return null;
            }

            previous = SelectObject(memoryDc, dib);
            if (previous == 0 || !DrawIconEx(memoryDc, 0, 0, icon, size, size, 0, 0, DiNormal))
            {
                return null;
            }

            if (!GdiFlush())
            {
                return null;
            }

            var pixels = new byte[size * size * 4];
            Marshal.Copy(bits, pixels, 0, pixels.Length);
            return new ShellIconImage(size, size, pixels);
        }
        finally
        {
            if (previous != 0 && memoryDc != 0)
            {
                SelectObject(memoryDc, previous);
            }

            if (dib != 0)
            {
                DeleteObject(dib);
            }

            if (memoryDc != 0)
            {
                DeleteDC(memoryDc);
            }

            ReleaseDC(0, screenDc);
        }
    }

    [DllImport("shell32.dll", CharSet = CharSet.Unicode)]
    private static extern nint SHGetFileInfo(
        string path,
        uint fileAttributes,
        out SHFILEINFO fileInfo,
        uint fileInfoSize,
        uint flags);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyIcon(nint icon);

    [DllImport("user32.dll")]
    private static extern nint GetDC(nint windowHandle);

    [DllImport("user32.dll")]
    private static extern int ReleaseDC(nint windowHandle, nint deviceContext);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DrawIconEx(
        nint deviceContext,
        int x,
        int y,
        nint icon,
        int width,
        int height,
        uint frame,
        nint brush,
        uint flags);

    [DllImport("gdi32.dll")]
    private static extern nint CreateCompatibleDC(nint deviceContext);

    [DllImport("gdi32.dll")]
    private static extern bool DeleteDC(nint deviceContext);

    [DllImport("gdi32.dll")]
    private static extern nint CreateDIBSection(
        nint deviceContext,
        ref BITMAPINFO bitmapInfo,
        uint usage,
        out nint bits,
        nint section,
        uint offset);

    [DllImport("gdi32.dll")]
    private static extern nint SelectObject(nint deviceContext, nint objectHandle);

    [DllImport("gdi32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GdiFlush();

    [DllImport("gdi32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteObject(nint objectHandle);

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct SHFILEINFO
    {
        public nint Icon;
        public int IconIndex;
        public uint Attributes;

        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        public string DisplayName;

        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 80)]
        public string TypeName;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAPINFO
    {
        public BITMAPINFOHEADER Header;
        public uint Color1;
        public uint Color2;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAPINFOHEADER
    {
        public uint Size;
        public int Width;
        public int Height;
        public ushort Planes;
        public ushort BitCount;
        public uint Compression;
        public uint ImageSize;
        public int XPelsPerMeter;
        public int YPelsPerMeter;
        public uint ColorsUsed;
        public uint ColorsImportant;
    }
}
