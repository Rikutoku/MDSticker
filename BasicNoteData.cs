using System;
using Windows.UI;

public class BasicNoteData
{
	public required string Path { get; set; }

	public int WindowX { get; set; } = 0;

	public int WindowY { get; set; }= 0;

	public int WindowWidth { get; set; } = 500;

	public int WindowHeight { get; set; } = 500;

    //public Color color { get; set; }
}
