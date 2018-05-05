package
{
	import flash.display.Sprite;
	import flash.events.Event;
	import skeletonsprint.Board;
	
	/**
	 * ...
	 * @author nano
	 */
	public class Main extends Sprite 
	{
		private var board:Board = new Board(800, Board.STD_BLOCK_LIB);
		public function Main() 
		{
			if (stage) init();
			else addEventListener(Event.ADDED_TO_STAGE, init);
			
			
		}
		
		private function initBoard():void{
			
		}
		
		private function init(e:Event = null):void 
		{
			removeEventListener(Event.ADDED_TO_STAGE, init);
			addChild(board);
		}
		
	}
	
}