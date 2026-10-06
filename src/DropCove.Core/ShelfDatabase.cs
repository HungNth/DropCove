using System.Globalization;
using Microsoft.Data.Sqlite;

namespace DropCove.Core;

internal sealed class ShelfDatabase(string databasePath)
{
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly string _connectionString = new SqliteConnectionStringBuilder
    {
        DataSource = databasePath,
        Mode = SqliteOpenMode.ReadWriteCreate,
        ForeignKeys = true,
        Pooling = false,
    }.ToString();

    public Task<ShelfDatabaseOpenResult> OpenAndLoadAsync(CancellationToken cancellationToken) =>
        RunAsync(OpenAndLoad, cancellationToken);

    public Task SaveRailPlacementAsync(
        ShelfRailPlacement placement,
        CancellationToken cancellationToken) =>
        RunAsync(() => SaveRailPlacement(placement), cancellationToken);

    public Task SaveSizingStateAsync(
        ShelfSizingState state,
        CancellationToken cancellationToken) =>
        RunAsync(() => SaveSizingState(state), cancellationToken);

    public Task AddBatchAsync(ShelfBatch batch, ShelfSizingState? sizingState, CancellationToken cancellationToken) =>
        RunAsync(() => AddBatch(batch, sizingState), cancellationToken);

    public Task SetPinnedAsync(Guid itemId, bool isPinned, CancellationToken cancellationToken) =>
        RunAsync(() => SetPinned(itemId, isPinned), cancellationToken);

    public Task SetAllItemsPinnedAsync(Guid batchId, bool isPinned, CancellationToken cancellationToken) =>
        RunAsync(() => SetAllItemsPinned(batchId, isPinned), cancellationToken);

    public Task RemoveItemsAsync(IReadOnlyCollection<Guid> itemIds, CancellationToken cancellationToken) =>
        RunAsync(() => RemoveItems(itemIds), cancellationToken);

    public Task RemoveBatchAsync(Guid batchId, CancellationToken cancellationToken) =>
        RunAsync(() => RemoveBatch(batchId), cancellationToken);

    public Task ClearTemporaryItemsAsync(CancellationToken cancellationToken) =>
        RunAsync(ClearTemporaryItems, cancellationToken);

    private async Task RunAsync(Action action, CancellationToken cancellationToken)
    {
        await _gate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            await Task.Run(action, cancellationToken).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }
    }

    private async Task<T> RunAsync<T>(Func<T> action, CancellationToken cancellationToken)
    {
        await _gate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            return await Task.Run(action, cancellationToken).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }
    }
    private ShelfDatabaseOpenResult OpenAndLoad()
    {
        var directory = Path.GetDirectoryName(databasePath);
        if (!string.IsNullOrEmpty(directory))
        {
            Directory.CreateDirectory(directory);
        }

        try
        {
            var batches = LoadBatches();
            var railPlacement = LoadRailPlacement();
            var sizingState = LoadSizingState(batches.Count > 0);
            return new ShelfDatabaseOpenResult(batches, railPlacement, sizingState, null);
        }
        catch (Exception exception) when (File.Exists(databasePath) && IsRecoverableDatabaseFailure(exception))
        {
            var backupPath = PreserveFailedDatabase();
            var batches = LoadBatches();
            var railPlacement = LoadRailPlacement();
            var sizingState = LoadSizingState(batches.Count > 0);
            return new ShelfDatabaseOpenResult(batches, railPlacement, sizingState, backupPath);
        }
    }

    private IReadOnlyList<ShelfBatch> LoadBatches()
    {
        using var connection = OpenConnection();
        ConfigureDatabase(connection);
        CreateSchema(connection);
        EnsureHealthy(connection);
        using (var transaction = connection.BeginTransaction())
        {
            DeleteEmptyBatches(connection, transaction);
            DeleteSizeWhenEmpty(connection, transaction);
            transaction.Commit();
        }

        var batches = new List<ShelfBatch>();
        using var batchCommand = connection.CreateCommand();
        batchCommand.CommandText = "SELECT id, created_at FROM shelf_batches ORDER BY position";
        using var batchReader = batchCommand.ExecuteReader();
        while (batchReader.Read())
        {
            var batchId = Guid.Parse(batchReader.GetString(0));
            var createdAt = DateTimeOffset.ParseExact(
                batchReader.GetString(1),
                "O",
                CultureInfo.InvariantCulture,
                DateTimeStyles.RoundtripKind);
            batches.Add(new ShelfBatch(batchId, createdAt, LoadItems(connection, batchId)));
        }

        return batches;
    }

    private string PreserveFailedDatabase()
    {
        var directory = Path.GetDirectoryName(databasePath) ?? string.Empty;
        var name = Path.GetFileNameWithoutExtension(databasePath);
        var extension = Path.GetExtension(databasePath);
        var timestamp = DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff", CultureInfo.InvariantCulture);
        var backupPath = Path.Combine(directory, $"{name}.corrupt-{timestamp}{extension}");
        for (var suffix = 2; File.Exists(backupPath); suffix++)
        {
            backupPath = Path.Combine(directory, $"{name}.corrupt-{timestamp}-{suffix}{extension}");
        }

        File.Move(databasePath, backupPath);
        MoveSidecar("-wal", backupPath);
        MoveSidecar("-shm", backupPath);
        return backupPath;
    }

    private void MoveSidecar(string suffix, string backupPath)
    {
        var sidecarPath = databasePath + suffix;
        if (File.Exists(sidecarPath))
        {
            File.Move(sidecarPath, backupPath + suffix);
        }
    }

    private static bool IsRecoverableDatabaseFailure(Exception exception) => exception is
        SqliteException or InvalidDataException or FormatException or InvalidCastException or IOException;

    private static void EnsureHealthy(SqliteConnection connection)
    {
        using var command = connection.CreateCommand();
        command.CommandText = "PRAGMA quick_check";
        if (!string.Equals(command.ExecuteScalar() as string, "ok", StringComparison.Ordinal))
        {
            throw new InvalidDataException("SQLite integrity check failed.");
        }
    }

    private void AddBatch(ShelfBatch batch, ShelfSizingState? sizingState)
    {
        using var connection = OpenConnection();
        using var transaction = connection.BeginTransaction();
        using var batchCommand = connection.CreateCommand();
        batchCommand.Transaction = transaction;
        batchCommand.CommandText =
            """
            INSERT INTO shelf_batches (id, created_at, position)
            VALUES ($id, $createdAt, COALESCE((SELECT MIN(position) - 1 FROM shelf_batches), 0))
            """;
        batchCommand.Parameters.AddWithValue("$id", batch.Id.ToString("D"));
        batchCommand.Parameters.AddWithValue("$createdAt", batch.CreatedAt.ToString("O", CultureInfo.InvariantCulture));
        batchCommand.ExecuteNonQuery();

        using var itemCommand = connection.CreateCommand();
        itemCommand.Transaction = transaction;
        itemCommand.CommandText =
            """
            INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
            VALUES ($id, $batchId, $position, $path, $name, $isFolder, $isPinned)
            """;
        var id = itemCommand.Parameters.Add("$id", SqliteType.Text);
        var batchId = itemCommand.Parameters.Add("$batchId", SqliteType.Text);
        var position = itemCommand.Parameters.Add("$position", SqliteType.Integer);
        var path = itemCommand.Parameters.Add("$path", SqliteType.Text);
        var name = itemCommand.Parameters.Add("$name", SqliteType.Text);
        var isFolder = itemCommand.Parameters.Add("$isFolder", SqliteType.Integer);
        var isPinned = itemCommand.Parameters.Add("$isPinned", SqliteType.Integer);
        for (var index = 0; index < batch.Items.Count; index++)
        {
            var item = batch.Items[index];
            id.Value = item.Id.ToString("D");
            batchId.Value = batch.Id.ToString("D");
            position.Value = index;
            path.Value = item.Path;
            name.Value = item.Name;
            isFolder.Value = item.IsFolder;
            isPinned.Value = item.IsPinned;
            itemCommand.ExecuteNonQuery();
        }

        if (sizingState is { } state)
        {
            SaveSizingState(connection, transaction, state);
        }

        transaction.Commit();
    }

    private void SetPinned(Guid itemId, bool isPinned)
    {
        using var connection = OpenConnection();
        using var command = connection.CreateCommand();
        command.CommandText = "UPDATE shelf_items SET is_pinned = $isPinned WHERE id = $id";
        command.Parameters.AddWithValue("$isPinned", isPinned);
        command.Parameters.AddWithValue("$id", itemId.ToString("D"));
        command.ExecuteNonQuery();
    }

    private void SetAllItemsPinned(Guid batchId, bool isPinned)
    {
        using var connection = OpenConnection();
        using var command = connection.CreateCommand();
        command.CommandText = "UPDATE shelf_items SET is_pinned = $isPinned WHERE batch_id = $batchId AND is_pinned <> $isPinned";
        command.Parameters.AddWithValue("$isPinned", isPinned);
        command.Parameters.AddWithValue("$batchId", batchId.ToString("D"));
        command.ExecuteNonQuery();
    }

    private void RemoveItems(IReadOnlyCollection<Guid> itemIds)
    {
        if (itemIds.Count == 0)
        {
            return;
        }

        using var connection = OpenConnection();
        using var transaction = connection.BeginTransaction();
        using var itemCommand = connection.CreateCommand();
        itemCommand.Transaction = transaction;
        itemCommand.CommandText = "DELETE FROM shelf_items WHERE id = $id";
        var id = itemCommand.Parameters.Add("$id", SqliteType.Text);
        foreach (var itemId in itemIds)
        {
            id.Value = itemId.ToString("D");
            itemCommand.ExecuteNonQuery();
        }

        DeleteEmptyBatches(connection, transaction);
        DeleteSizeWhenEmpty(connection, transaction);
        transaction.Commit();
    }

    private void RemoveBatch(Guid batchId)
    {
        using var connection = OpenConnection();
        using var transaction = connection.BeginTransaction();
        using var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText = "DELETE FROM shelf_batches WHERE id = $id";
        command.Parameters.AddWithValue("$id", batchId.ToString("D"));
        command.ExecuteNonQuery();
        DeleteSizeWhenEmpty(connection, transaction);
        transaction.Commit();
    }

    private void ClearTemporaryItems()
    {
        using var connection = OpenConnection();
        using var transaction = connection.BeginTransaction();
        using var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText = "DELETE FROM shelf_items WHERE is_pinned = 0";
        command.ExecuteNonQuery();
        DeleteEmptyBatches(connection, transaction);
        DeleteSizeWhenEmpty(connection, transaction);
        transaction.Commit();
    }

    private static void DeleteEmptyBatches(SqliteConnection connection, SqliteTransaction transaction)
    {
        using var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText =
            "DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)";
        command.ExecuteNonQuery();
    }

    private static void DeleteSizeWhenEmpty(SqliteConnection connection, SqliteTransaction transaction)
    {
        using var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText = "DELETE FROM shelf_settings WHERE NOT EXISTS (SELECT 1 FROM shelf_items)";
        command.ExecuteNonQuery();
    }
    private SqliteConnection OpenConnection()
    {
        var connection = new SqliteConnection(_connectionString);
        try
        {
            connection.Open();
            return connection;
        }
        catch
        {
            connection.Dispose();
            throw;
        }
    }

    private static void ConfigureDatabase(SqliteConnection connection)
    {
        using var command = connection.CreateCommand();
        command.CommandText = "PRAGMA journal_mode = WAL";
        command.ExecuteNonQuery();
    }

    private static void CreateSchema(SqliteConnection connection)
    {
        using var command = connection.CreateCommand();
        command.CommandText =
            """
            CREATE TABLE IF NOT EXISTS shelf_batches (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                position INTEGER NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS shelf_items (
                id TEXT PRIMARY KEY,
                batch_id TEXT NOT NULL REFERENCES shelf_batches(id) ON DELETE CASCADE,
                position INTEGER NOT NULL,
                path TEXT NOT NULL,
                name TEXT NOT NULL,
                is_folder INTEGER NOT NULL CHECK (is_folder IN (0, 1)),
                is_pinned INTEGER NOT NULL CHECK (is_pinned IN (0, 1)),
                UNIQUE (batch_id, position)
            );
            CREATE TABLE IF NOT EXISTS shelf_ui_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                rail_monitor TEXT NOT NULL,
                rail_edge TEXT NOT NULL CHECK (rail_edge IN ('Left', 'Right'))
            );
            INSERT OR IGNORE INTO shelf_ui_state (id, rail_monitor, rail_edge)
            VALUES (1, '', 'Right');
            CREATE TABLE IF NOT EXISTS shelf_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                preferred_width INTEGER NOT NULL,
                manual_height INTEGER NULL
            );
            """;
        command.ExecuteNonQuery();
        using var columns = connection.CreateCommand();
        columns.CommandText = "SELECT COUNT(*) FROM pragma_table_info('shelf_settings') WHERE name = 'preferred_height'";
        if ((long)columns.ExecuteScalar()! > 0)
        {
            using var transaction = connection.BeginTransaction();
            using var cutover = connection.CreateCommand();
            cutover.Transaction = transaction;
            cutover.CommandText =
                """
                ALTER TABLE shelf_settings RENAME TO old_shelf_settings;
                CREATE TABLE shelf_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    preferred_width INTEGER NOT NULL,
                    manual_height INTEGER NULL
                );
                INSERT INTO shelf_settings SELECT id, preferred_width, preferred_height
                FROM old_shelf_settings
                WHERE preferred_width >= 180 AND preferred_height >= 180
                  AND EXISTS (SELECT 1 FROM shelf_items);
                DROP TABLE old_shelf_settings;
                """;
            cutover.ExecuteNonQuery();
            transaction.Commit();
        }
    }

    private ShelfRailPlacement LoadRailPlacement()
    {
        using var connection = OpenConnection();
        using var command = connection.CreateCommand();
        command.CommandText = "SELECT rail_monitor, rail_edge FROM shelf_ui_state WHERE id = 1";
        using var reader = command.ExecuteReader();
        if (!reader.Read())
        {
            return ShelfRailPlacement.Default;
        }

        var edge = Enum.TryParse<ShelfRailEdge>(reader.GetString(1), ignoreCase: true, out var parsed)
            ? parsed
            : ShelfRailEdge.Right;
        return new ShelfRailPlacement(reader.GetString(0), edge);
    }

    private void SaveRailPlacement(ShelfRailPlacement placement)
    {
        ArgumentNullException.ThrowIfNull(placement);
        if (placement.Edge is not (ShelfRailEdge.Left or ShelfRailEdge.Right))
        {
            throw new ArgumentOutOfRangeException(nameof(placement));
        }

        using var connection = OpenConnection();
        using var command = connection.CreateCommand();
        command.CommandText =
            """
            INSERT INTO shelf_ui_state (id, rail_monitor, rail_edge)
            VALUES (1, $monitor, $edge)
            ON CONFLICT(id) DO UPDATE SET
                rail_monitor = excluded.rail_monitor,
                rail_edge = excluded.rail_edge
            """;
        command.Parameters.AddWithValue("$monitor", placement.MonitorId);
        command.Parameters.AddWithValue("$edge", placement.Edge.ToString());
        command.ExecuteNonQuery();
    }

    private ShelfSizingState LoadSizingState(bool hasBatches)
    {
        using var connection = OpenConnection();
        using var command = connection.CreateCommand();
        command.CommandText = "SELECT preferred_width, manual_height FROM shelf_settings WHERE id = 1";
        using var reader = command.ExecuteReader();
        if (!reader.Read())
        {
            return ShelfSizingState.Default;
        }

        var state = new ShelfSizingState(reader.GetInt32(0), reader.IsDBNull(1) ? null : reader.GetInt32(1));
        if (!hasBatches || !state.IsValid)
        {
            reader.Close();
            using var deleteCommand = connection.CreateCommand();
            deleteCommand.CommandText = "DELETE FROM shelf_settings WHERE id = 1";
            deleteCommand.ExecuteNonQuery();
            return ShelfSizingState.Default;
        }

        return state;
    }

    private void SaveSizingState(ShelfSizingState state)
    {
        using var connection = OpenConnection();
        SaveSizingState(connection, null, state);
    }

    private static void SaveSizingState(SqliteConnection connection, SqliteTransaction? transaction, ShelfSizingState state)
    {
        using var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText =
            """
            INSERT INTO shelf_settings (id, preferred_width, manual_height)
            VALUES (1, $width, $height)
            ON CONFLICT(id) DO UPDATE SET
                preferred_width = excluded.preferred_width,
                manual_height = excluded.manual_height
            """;
        command.Parameters.AddWithValue("$width", state.PreferredWidth);
        command.Parameters.AddWithValue("$height", (object?)state.ManualHeightOverride ?? DBNull.Value);
        command.ExecuteNonQuery();
    }


    private static IReadOnlyList<ShelfItem> LoadItems(SqliteConnection connection, Guid batchId)
    {
        var items = new List<ShelfItem>();
        using var command = connection.CreateCommand();
        command.CommandText =
            """
            SELECT id, path, name, is_folder, is_pinned
            FROM shelf_items
            WHERE batch_id = $batchId
            ORDER BY position
            """;
        command.Parameters.AddWithValue("$batchId", batchId.ToString("D"));
        using var reader = command.ExecuteReader();
        while (reader.Read())
        {
            items.Add(new ShelfItem(
                Guid.Parse(reader.GetString(0)),
                reader.GetString(1),
                reader.GetString(2),
                reader.GetBoolean(3),
                reader.GetBoolean(4)));
        }

        return items;
    }
}

internal sealed record ShelfDatabaseOpenResult(
    IReadOnlyList<ShelfBatch> Batches,
    ShelfRailPlacement RailPlacement,
    ShelfSizingState SizingState,
    string? RecoveryBackupPath);
