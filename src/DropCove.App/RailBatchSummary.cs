using System.ComponentModel;
using DropCove.Core;
using Microsoft.UI.Xaml;

namespace DropCove;

internal sealed class RailBatchSummary : INotifyPropertyChanged
{
    private static readonly PropertyChangedEventArgs AllPropertiesChanged = new(string.Empty);
    private static readonly PropertyChangedEventArgs ActionsEnabledChanged = new(nameof(ActionsEnabled));
    private bool _isBusy;
    private bool _hasFollowingBatch;

    public RailBatchSummary(ShelfBatch batch, bool hasFollowingBatch)
    {
        Batch = batch;
        _hasFollowingBatch = hasFollowingBatch;
        Presentation = ShelfPresentation.CreateBatchCard(batch);
    }

    public event PropertyChangedEventHandler? PropertyChanged;
    public ShelfBatch Batch { get; private set; }
    public BatchCardViewModel Presentation { get; }
    public bool ActionsEnabled => !_isBusy;
    public Thickness RowMargin => new(0, 0, 0, _hasFollowingBatch ? EdgeRailSizingPolicy.Gap : 0);
    public string RemoveAutomationName => Presentation.IsSingleItem ? $"Remove {Presentation.Title}" : Presentation.RemoveBatchAutomationName;

    public void Update(ShelfBatch batch, bool hasFollowingBatch)
    {
        var batchChanged = !ReferenceEquals(Batch, batch);
        var spacingChanged = _hasFollowingBatch != hasFollowingBatch;
        Batch = batch;
        _hasFollowingBatch = hasFollowingBatch;
        if (batchChanged) ShelfPresentation.CreateBatchCard(batch, Presentation);
        if (batchChanged || spacingChanged) PropertyChanged?.Invoke(this, AllPropertiesChanged);
    }

    public void SetBusy(bool isBusy)
    {
        if (_isBusy == isBusy) return;
        _isBusy = isBusy;
        PropertyChanged?.Invoke(this, ActionsEnabledChanged);
    }
}
