#include <SFML/Window.hpp> 
#include <SFML/Graphics.hpp>
#include <vector>

namespace genericTD{
	class UIObject : public sf::Drawable {
		public:
			std::vector<UIObject> children;
			sf::Drawable* self = NULL;
			UIObject(){
				
			}
			UIObject(sf::CircleShape s){
				self = &s;
			}
		private:
			void draw(sf::RenderTarget& target, sf::RenderStates states) const{
				if(self != NULL){ //May be NULL
					target.draw(*self);
				}
				for(int i = 0; i != children.size(); i++){
					//target.draw();
				}
			}
	};
	struct UserInput{
		
	};
	struct LogicLayer{
		
	};
	struct Renderer{
		std::vector <UIObject> children;
		void draw(sf::RenderWindow* win){
			for(int i = 0;  i != children.size(); i++){
				win->draw(children.at(i));
			}
			return;
		}
		addChild(sf::CircleShape s){
			children.push_back(UIObject()); //TODO: construct with s
		}
	};
	struct Game{
			UserInput input;
			LogicLayer logic;
			Renderer renderer;
	};
}