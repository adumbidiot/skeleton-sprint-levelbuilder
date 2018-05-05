package skeletonsprint 
{
	import flash.utils.Dictionary;
	import flash.display.Bitmap;
	/**
	 * ...
	 * @author nano
	 */
	public class STDBlockLibrary 
	{
		[Embed(source = "STDBlockLibraryAssets/backgrounds/M0.png")]
		static private var M0:Class;
		
		static public const sprites:Dictionary = new Dictionary();
		public function STDBlockLibrary() 
		{
			sprites["M0"] = (M0 as Bitmap); 
			sprites["default_bg"] = static["M0"];
		}
		
	}

}