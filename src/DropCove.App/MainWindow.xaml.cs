using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>
/// Hosts the Drop Shelf page.
/// </summary>
public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        var iconPath = System.IO.Path.Combine(System.AppContext.BaseDirectory, "Assets", "AppIcon.ico");
        if (System.IO.File.Exists(iconPath))
        {
            AppWindow.SetIcon(iconPath);
        }
        RootFrame.Navigate(typeof(MainPage));
    }
}
