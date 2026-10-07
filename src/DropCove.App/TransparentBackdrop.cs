using System.Runtime.InteropServices;
using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Media;

namespace DropCove;

/// <summary>Replaces the opaque default window surface with a fully transparent one.</summary>
/// <remarks>
/// The host window must already be prepared with <see cref="DropCove.Native.WindowInterop.EnableTransparentSurface"/>
/// and must answer <c>WM_ERASEBKGND</c> with <see cref="DropCove.Native.WindowInterop.ClearTransparentBackground"/>.
/// XAML content then supplies the visible surface, including antialiased rounded corners.
/// </remarks>
internal sealed class TransparentBackdrop : SystemBackdrop
{
    private static nint s_dispatcherQueueController;
    private static Windows.UI.Composition.Compositor? s_compositor;
    private Windows.UI.Composition.CompositionColorBrush? _brush;

    protected override void OnTargetConnected(ICompositionSupportsSystemBackdrop connectedTarget, XamlRoot xamlRoot)
    {
        base.OnTargetConnected(connectedTarget, xamlRoot);
        _brush = GetCompositor().CreateColorBrush(Windows.UI.Color.FromArgb(0, 0, 0, 0));
        connectedTarget.SystemBackdrop = _brush;
    }

    protected override void OnTargetDisconnected(ICompositionSupportsSystemBackdrop disconnectedTarget)
    {
        disconnectedTarget.SystemBackdrop = null;
        _brush?.Dispose();
        _brush = null;
        base.OnTargetDisconnected(disconnectedTarget);
    }

    // ICompositionSupportsSystemBackdrop accepts only Windows.UI.Composition brushes, whose
    // compositor requires a Windows.System.DispatcherQueue on the UI thread.
    private static Windows.UI.Composition.Compositor GetCompositor()
    {
        if (s_compositor is not null)
        {
            return s_compositor;
        }

        if (Windows.System.DispatcherQueue.GetForCurrentThread() is null && s_dispatcherQueueController == 0)
        {
            var options = new DispatcherQueueOptions
            {
                Size = Marshal.SizeOf<DispatcherQueueOptions>(),
                ThreadType = 2, // DQTYPE_THREAD_CURRENT
                ApartmentType = 2, // DQTAT_COM_STA
            };
            Marshal.ThrowExceptionForHR(CreateDispatcherQueueController(options, out s_dispatcherQueueController));
        }

        return s_compositor = new Windows.UI.Composition.Compositor();
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct DispatcherQueueOptions
    {
        public int Size;
        public int ThreadType;
        public int ApartmentType;
    }

    [DllImport("CoreMessaging.dll")]
    private static extern int CreateDispatcherQueueController(DispatcherQueueOptions options, out nint dispatcherQueueController);
}
