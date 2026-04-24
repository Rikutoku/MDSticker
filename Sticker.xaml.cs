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
using System.Runtime.InteropServices.WindowsRuntime;
using System.Numerics;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Windows.UI.ViewManagement;

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

            //OverlappedPresenter presenter = OverlappedPresenter.Create();
            //presenter.IsResizable = true;
            //presenter.IsMaximizable = false;
            //presenter.IsMinimizable = false;
            //AppWindow.SetPresenter(presenter);

            this.ExtendsContentIntoTitleBar = true;
            this.SetTitleBar(customTitlebar);

            AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Collapsed;
            AppWindow.Resize(new Windows.Graphics.SizeInt32(markdownFile.WindowWidth, markdownFile.WindowHeight));
            AppWindow.Move(new Windows.Graphics.PointInt32(markdownFile.WindowX, markdownFile.WindowY));

            if (!string.IsNullOrEmpty(markdownFile.Path))
            {
                this.MarkdownViewer.Text = System.IO.File.ReadAllText(markdownFile.Path);
            }
        }

        private void AddButton_Click(object sender, RoutedEventArgs e)
        {
            
        }

        private void customTitlebar_PointerEntered(object sender, PointerRoutedEventArgs e)
        {
            customTitlebarContent.Translation =new Vector3(0, 0, 0);
        }

        private void customTitlebar_PointerExited(object sender, PointerRoutedEventArgs e)
        {
            customTitlebarContent.Translation = new Vector3(0, -32, 0);
        }
    }
}