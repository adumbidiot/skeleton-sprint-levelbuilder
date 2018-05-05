package skeletonsprint 
{
	import flash.display.Bitmap;
	import flash.display.Loader;
	import flash.display.Sprite;
	import flash.events.Event;
	import flash.events.IOErrorEvent;
	import flash.events.ProgressEvent;
	import flash.utils.Dictionary;
	/**
	 * ...
	 * @author nano
	 */
	public class Board extends Sprite
	{
		//CONSTANTS
		public static const NO_BLOCK_LIB:int = 0;
		public static const STD_BLOCK_LIB:int = 1;
		//END CONSTANTS
		
		private var blocks:Dictionary;
		
		public function Board(width:int = 1280, lib:int = NO_BLOCK_LIB) 
		{
			this.width = width;
			this.height = width * 16/9;
			if (STD_BLOCK_LIB){
				loadSTDBlockLib();
			}
		}
		
		public function loadSTDBlockLib():void{
			for (var key:Object in STDBlockLibrary.sprites){
				trace(key.toString());
			}
			trace("[DEBUG] Loaded Standard Block Library");
		}
		
		public function defineBlock(lbl:String, img:Bitmap):void{
			blocks[lbl] = img;
		}
		
		public function removeBlock(lbl:String): void{
			//TODO: Implement
		}
		public function update(): void{
			
		}
	}

}