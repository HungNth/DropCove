using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>Hosts the collapsed Edge Rail overlay.</summary>
public sealed partial class EdgeRailWindow : Window
{
    private const int RailWidth = 64;
    private readonly nint _windowHandle;
    private readonly Action _openShelf;

    /// <summary>Creates a collapsed Edge Rail for the current Shelf Batches.</summary>
    /// <param name="batches">The batches represented by the rail.</param>
    /// <param name="openShelf">Opens and activates the unified Drop Shelf.</param>
    public EdgeRailWindow(IReadOnlyList<ShelfBatch> batches, Action openShelf)
    {
        ArgumentNullException.ThrowIfNull(batches);
        ArgumentNullException.ThrowIfNull(openShelf);

        InitializeComponent();
        _windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WindowInterop.MakeBorderless(_windowHandle);
        _openShelf = openShelf;
        UpdateBatches(batches);
    }

    /// <summary>Gets the native rail window handle.</summary>
    public nint WindowHandle => _windowHandle;

    /// <summary>Rebuilds the compact batch summaries.</summary>
    /// <param name="batches">The current Shelf Batches.</param>
    public void UpdateBatches(IReadOnlyList<ShelfBatch> batches)
    {
        ArgumentNullException.ThrowIfNull(batches);
        BatchList.ItemsSource = batches.Select(CreateBatchSummary).ToArray();
    }

    /// <summary>Positions and shows the rail without activating DropCove.</summary>
    /// <param name="placement">The remembered monitor and edge.</param>
    public void Show(ShelfRailPlacement placement)
    {
        ArgumentNullException.ThrowIfNull(placement);
        WindowInterop.PositionEdgeRail(
            _windowHandle,
            placement.MonitorId,
            placement.Edge == ShelfRailEdge.Left,
            RailWidth);
        WindowInterop.ShowNoActivate(_windowHandle);
    }

    /// <summary>Hides the rail window without destroying it.</summary>
    public void Hide() => WindowInterop.Hide(_windowHandle);


    private void OnOpenShelfClicked(object sender, RoutedEventArgs e) => _openShelf();

    private static RailBatchSummary CreateBatchSummary(ShelfBatch batch)
    {
        var first = batch.Items[0];
        var title = batch.Items.Count == 1
            ? first.Name
            : $"{batch.Items.Count} items";
        var subtitle = batch.Items.Count == 1
            ? first.IsFolder ? "Folder" : "File"
            : first.Name;
        var pinnedGlyph = batch.Items.Any(item => item.IsPinned) ? "\uE718" : string.Empty;
        return new RailBatchSummary(
            title,
            subtitle,
            first.IsFolder ? "\uE8B7" : "\uE8A5",
            pinnedGlyph,
            $"Edge Rail batch {title}");
    }

}

internal sealed record RailBatchSummary(
    string Title,
    string Subtitle,
    string Glyph,
    string PinnedGlyph,
    string AutomationName);
