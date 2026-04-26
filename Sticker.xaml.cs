using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Numerics;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading.Tasks;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.UI.ViewManagement;
using WinRT.Interop;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace MDSticker
{
    /// <summary>
    /// An empty window that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class Sticker : Window
    {
        private readonly BasicNoteData _markdownFile;

        private DispatcherTimer _updateFrameTimer = new DispatcherTimer();

        public Sticker(BasicNoteData noteData)
        {
            InitializeComponent();


            _markdownFile = noteData;

            //OverlappedPresenter presenter = OverlappedPresenter.Create();
            //presenter.IsResizable = true;
            //presenter.IsMaximizable = false;
            //presenter.IsMinimizable = false;
            //AppWindow.SetPresenter(presenter);

            ExtendsContentIntoTitleBar = true;
            SetTitleBar(customTitlebar);

            AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Collapsed;
            AppWindow.Resize(new Windows.Graphics.SizeInt32(_markdownFile.WindowWidth, _markdownFile.WindowHeight));
            AppWindow.Move(new Windows.Graphics.PointInt32(_markdownFile.WindowX, _markdownFile.WindowY));

            _updateFrameTimer.Interval = TimeSpan.FromSeconds(1);
            _updateFrameTimer.Tick += UpdateFrameTimer_Tick;

            AppWindow.Changed += AppWindow_Changed;

            AppWindow.Closing += AppWindow_Closing;
        }

        private async void AppWindow_Closing(AppWindow sender, AppWindowClosingEventArgs args)
        {

            //bad thing but easy part 1
            StickerManager.AmountOfStickers--;

            if (StickerManager.AmountOfStickers == 0)
            {
                await StickerManager.SaveSettingsAsync();
            }
            //end of it
        }

        private void AppWindow_Changed(AppWindow sender, AppWindowChangedEventArgs args)
        {
            if (args.DidPositionChange || args.DidSizeChange)
            {
                _updateFrameTimer.Stop();
                _updateFrameTimer.Start();
            }
        }

        private async void UpdateFrameTimer_Tick(object? sender, object e)
        {
            _updateFrameTimer.Stop();
            _markdownFile.WindowX = AppWindow.Position.X;
            _markdownFile.WindowY = AppWindow.Position.Y;
            _markdownFile.WindowWidth = AppWindow.Size.Width;
            _markdownFile.WindowHeight = AppWindow.Size.Height;
            await StickerManager.UpdateNoteAsync(_markdownFile);
        }

        private async void RootGrid_Loaded(object sender, RoutedEventArgs e)
        {
            if (string.IsNullOrEmpty(_markdownFile.Path))
            {
                _markdownFile.Path = await PickMarkdownFileAsync();

                if (string.IsNullOrEmpty(_markdownFile.Path))
                {
                    Close();
                    return;
                }

                if(!await StickerManager.AddNoteAsync(_markdownFile))
                {
                    Close();
                    return;
                }
            }
            MarkdownViewer.Text = System.IO.File.ReadAllText(_markdownFile.Path);

            StickerManager.AmountOfStickers++;//bad thing but easy part 2
        }

        private async Task<string> PickMarkdownFileAsync()
        {
            FileOpenPicker openPicker = new FileOpenPicker();
            InitializeWithWindow.Initialize(openPicker, WindowNative.GetWindowHandle(this));

            //options
            openPicker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;
            openPicker.FileTypeFilter.Add(".md");

            return (await openPicker.PickSingleFileAsync()).Path;
        }

        private void customTitlebar_PointerEntered(object sender, PointerRoutedEventArgs e)
        {
            customTitlebarContent.Translation =new Vector3(0, 0, 0);
        }

        private void customTitlebar_PointerExited(object sender, PointerRoutedEventArgs e)
        {
            customTitlebarContent.Translation = new Vector3(0, -32, 0);
        }

        private async void CloseSticker_Click(object sender, RoutedEventArgs e)
        {
            if (!string.IsNullOrEmpty(_markdownFile.Path))
            {
                await StickerManager.RemoveNoteAsync(_markdownFile);
            }
            Close();
        }

        private async void OpenSticker_Click(object sender, RoutedEventArgs e)
        {
            new Sticker(new BasicNoteData()).Activate();
        }
    }
}