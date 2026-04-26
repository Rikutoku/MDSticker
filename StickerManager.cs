using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;

namespace MDSticker
{
    public static class StickerManager
    {
        public static int AmountOfStickers { get; set; } = 0;//need to be changed to something better

        private static readonly string _settingsPath = Path.Combine(AppContext.BaseDirectory, "noteList.json");

        private static readonly List<BasicNoteData> _notes;

        public static IReadOnlyList<BasicNoteData> GetAllNotes() => _notes;

        static StickerManager()
        {
            _notes = LoadNoteSettings();
        }

        public static int Count() => _notes.Count;

        private static List<BasicNoteData> LoadNoteSettings()
        {
            if (!File.Exists(_settingsPath))
            {
                File.WriteAllText(_settingsPath, JsonSerializer.Serialize(new List<BasicNoteData>()));
            }

            var json = File.ReadAllText(_settingsPath);

            return JsonSerializer.Deserialize<List<BasicNoteData>>(json) ?? new List<BasicNoteData>();
        }

        public static async Task SaveSettingsAsync()
        {
            var tmp = _settingsPath + ".tmp";
            await File.WriteAllTextAsync(tmp, JsonSerializer.Serialize(_notes));
            File.Move(tmp, _settingsPath, overwrite: true);
        }

        //true if added, false if already exists
        public static async Task<bool> AddNoteAsync(BasicNoteData note)
        {
            foreach (var existingNote in _notes)
            {
                if (existingNote.Path == note.Path) return false;
            }

            _notes.Add(note);
            await SaveSettingsAsync();
            return true;
        }

        public static async Task RemoveNoteAsync(BasicNoteData note)
        {
            _notes.Remove(note);
            await SaveSettingsAsync();

            //foreach (var note in _notes)
            //{
            //    if (note.Path == path)
            //    {
            //        _notes.Remove(note);
            //        await SaveSettingsAsync();
            //        return;
            //    }
            //}
        }

        public static async Task UpdateNoteAsync(BasicNoteData note)
        {
            foreach (var existingNote in _notes)
            {
                if (existingNote.Path == note.Path)
                {
                    existingNote.WindowX = note.WindowX;
                    existingNote.WindowY = note.WindowY;
                    existingNote.WindowWidth = note.WindowWidth;
                    existingNote.WindowHeight = note.WindowHeight;
                    return;
                }
            }
        }
    }
}
