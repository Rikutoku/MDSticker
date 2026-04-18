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
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace MDSticker
{
    /// <summary>
    /// An empty window that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class Sticker : Window
    {
        public Sticker(BasicNoteData markdownFile)
        {
            InitializeComponent();

            OverlappedPresenter presenter = OverlappedPresenter.Create();
            presenter.IsResizable = true;
            presenter.IsMaximizable = false;
            presenter.IsMinimizable = false;
            AppWindow.SetPresenter(presenter);
            ExtendsContentIntoTitleBar = true;

            AppWindow.Resize(new Windows.Graphics.SizeInt32(markdownFile.windowWidth, markdownFile.windowHeight));
            AppWindow.Move(new Windows.Graphics.PointInt32(markdownFile.windowX, markdownFile.windowY));

            if (!string.IsNullOrEmpty(markdownFile.path))
            {
                this.MarkdownViewer.Text = System.IO.File.ReadAllText(markdownFile.path);
            }
        }

        private void CloseButton_Click(object sender, WindowEventArgs e)
        {

        }

        private void AddButton_Click(object sender, RoutedEventArgs e)
        {
            //App.AddWindow();
        }
    }
}
