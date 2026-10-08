using System.Collections;
using System.Collections.Specialized;
using DropCove.Core;

namespace DropCove;

// ItemsRepeater uses IList for indexed access without enumerating an entire Shelf Batch.
internal sealed class ShelfItemProjectionList : IList, INotifyCollectionChanged
{
    private readonly Dictionary<Guid, ShelfItemViewModel> _materialized = [];
    private IReadOnlyList<ShelfItem> _items = Array.Empty<ShelfItem>();
    private static readonly NotifyCollectionChangedEventArgs Reset = new(NotifyCollectionChangedAction.Reset);

    public event NotifyCollectionChangedEventHandler? CollectionChanged;
    public int Count => _items.Count;
    public bool IsReadOnly => true;
    public bool IsFixedSize => true;
    public bool IsSynchronized => false;
    public object SyncRoot => this;
    public Dictionary<Guid, ShelfItemViewModel>.ValueCollection MaterializedItems => _materialized.Values;

    public object? this[int index]
    {
        get
        {
            var item = _items[index];
            if (!_materialized.TryGetValue(item.Id, out var projection))
            {
                projection = ShelfPresentation.CreateShelfItemViewModel(item);
                _materialized.Add(item.Id, projection);
            }
            return projection;
        }
        set => throw new NotSupportedException();
    }

    public void Update(IReadOnlyList<ShelfItem> items)
    {
        var shapeChanged = _items.Count != items.Count;
        if (!shapeChanged)
            for (var index = 0; index < items.Count; index++)
                if (_items[index].Id != items[index].Id) { shapeChanged = true; break; }
        _items = items;
        foreach (var item in items)
            if (_materialized.TryGetValue(item.Id, out var projection)) projection.Update(item);
        if (shapeChanged) CollectionChanged?.Invoke(this, Reset);
    }

    public void Release(ShelfItemViewModel item)
    {
        if (_materialized.TryGetValue(item.Item.Id, out var current) && ReferenceEquals(current, item))
            _materialized.Remove(item.Item.Id);
    }

    public void ClearPresentation()
    {
        _items = Array.Empty<ShelfItem>();
        _materialized.Clear();
        CollectionChanged?.Invoke(this, Reset);
    }

    public int IndexOf(object? value)
    {
        if (value is not ShelfItemViewModel projection) return -1;
        for (var index = 0; index < _items.Count; index++)
            if (_items[index].Id == projection.Item.Id) return index;
        return -1;
    }

    public bool Contains(object? value) => IndexOf(value) >= 0;

    public void CopyTo(Array array, int index)
    {
        for (var itemIndex = 0; itemIndex < Count; itemIndex++) array.SetValue(this[itemIndex], index + itemIndex);
    }

    public IEnumerator GetEnumerator()
    {
        for (var index = 0; index < Count; index++) yield return this[index];
    }

    int IList.Add(object? value) => throw new NotSupportedException();
    void IList.Clear() => throw new NotSupportedException();
    void IList.Insert(int index, object? value) => throw new NotSupportedException();
    void IList.Remove(object? value) => throw new NotSupportedException();
    void IList.RemoveAt(int index) => throw new NotSupportedException();
}
