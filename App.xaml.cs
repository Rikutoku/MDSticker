using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using Microsoft.UI.Xaml.Shapes;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading.Tasks;
using Windows.ApplicationModel;
using Windows.ApplicationModel.Activation;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Windows.Storage;
using WinRT.Interop;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace MDSticker
{
    /// <summary>
    /// Provides application-specific behavior to supplement the default Application class.
    /// </summary>
    public partial class App : Application
    {
        private Window? _window;

        private readonly List<Window> _openWindows = new();

        private NoteDataProvider noteDataProvider = new NoteDataProvider();


        /// <summary>
        /// Initializes the singleton application object.  This is the first line of authored code
        /// executed, and as such is the logical equivalent of main() or WinMain().
        /// </summary>
        public App()
        {
            InitializeComponent();
            noteDataProvider.LoadNoteList();
        }

        /// <summary>
        /// Invoked when the application is launched.
        /// </summary>
        /// <param name="args">Details about the launch request and process.</param>
        protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
        {
            if(noteDataProvider.Notes.Count == 0)
            {
                await AddNoteWindow();
            }

            foreach (var note in noteDataProvider.Notes)
            {
                CreateStickerWindow(note);
            }
        }

        private void CreateStickerWindow(BasicNoteData noteData)
        {
            var window = new Sticker(noteData);
            _openWindows.Add(window);
            window.Closed += (s, e) => _openWindows.Remove(window);
            window.Activate();
        }

        public async Task AddNoteWindow(string notePath)
        {
            var newNote = await noteDataProvider.AddNoteAsync(notePath);
            CreateStickerWindow(newNote);
        }

        public async Task CloseWindow(string notePath)
        {
            await noteDataProvider.RemoveNoteByPathAsync(notePath);
        }
    }
}
