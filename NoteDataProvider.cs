using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Windows.ApplicationModel.Preview.Notes;

namespace MDSticker
{
    internal class NoteDataProvider
    {
        private static readonly string SettingsPath = Path.Combine(AppContext.BaseDirectory, "settings.json");

        public List<BasicNoteData> Notes { get; private set; } = new List<BasicNoteData>();

        public void LoadNoteList()//no async because we want to load data before the app starts
        {
            if (!File.Exists(SettingsPath)) return;

            var json = File.ReadAllText(SettingsPath);
            Notes = JsonSerializer.Deserialize<List<BasicNoteData>>(json) ?? new List<BasicNoteData>();
        }

        private async Task SaveSettingsAsync()
        {
            var tmp = SettingsPath + ".tmp";
            await File.WriteAllTextAsync(tmp, JsonSerializer.Serialize(Notes));
            File.Move(tmp, SettingsPath, overwrite: true);
        }

        public async Task RemoveNoteByPathAsync(string path)
        {

            foreach (var note in Notes)
            {
                if (note.path == path)
                {
                    Notes.Remove(note);
                    await SaveSettingsAsync();
                    return;
                }
            }
        }

        public async Task<BasicNoteData> AddNoteAsync(string path)
        {
            foreach (var existingNote in Notes)
            {
                if (existingNote.path == path) return existingNote;
            }

            Notes.Add(new BasicNoteData { path = path });
            await SaveSettingsAsync();
            return Notes.Last();
        }

        //public void ChangeColor(string path, Windows.UI.Color color)
        //{
        //    foreach (var note in Notes)
        //    {
        //        if (note.path == path)
        //        {
        //            note.color = color;
        //            break;
        //        }
        //    }
        //}
    }
}
