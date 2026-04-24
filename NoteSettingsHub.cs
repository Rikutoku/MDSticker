using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;

namespace MDSticker
{
    public static class NoteSettingsHub
    {
        private static readonly string _settingsPath = Path.Combine(AppContext.BaseDirectory, "settings.json");

        private static readonly List<BasicNoteData> _notes;

        public static IReadOnlyList<BasicNoteData> GetAll() => _notes;

        static NoteSettingsHub()
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

        public static async Task AddNoteAsync(string path)
        {
            foreach (var existingNote in _notes)
            {
                if (existingNote.Path == path) return;
            }

            _notes.Add(new BasicNoteData { Path = path });
            await SaveSettingsAsync();
        }

        public static async Task RemoveNoteAsync(string path)
        {
            foreach (var note in _notes)
            {
                if (note.Path == path)
                {
                    _notes.Remove(note);
                    await SaveSettingsAsync();
                    return;
                }
            }
        }
        public static async Task RemoveNoteAsync(BasicNoteData noteTarget)
        {
            foreach (var note in _notes)
            {
                if (note.Path == noteTarget.Path)
                {
                    _notes.Remove(note);
                    await SaveSettingsAsync();
                    return;
                }
            }
        }
    }
}
