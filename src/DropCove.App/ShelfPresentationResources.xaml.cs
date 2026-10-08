using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>Provides compiled visual templates shared by the Drop Shelf and Edge Rail.</summary>
public sealed partial class ShelfPresentationResources : ResourceDictionary
{
    /// <summary>Loads the shared preview and Bulk Pinning templates.</summary>
    public ShelfPresentationResources() => InitializeComponent();
}
