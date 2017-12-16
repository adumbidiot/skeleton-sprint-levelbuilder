//#define SFML_STATIC
#include <SFML/Window.hpp>
#include <SFML/Graphics.hpp> 
#include <iostream>
#include "logicLayer.h"


int main(){
	sf::RenderWindow win(sf::VideoMode(1920, 1080), "Generic TowerDefense");
	genericTD::Game gtd;
	gtd.renderer.addChild(sf::CircleShape(80, 4));
	
	while(win.isOpen()){
		sf::Event event;
		while(win.pollEvent(event)){
			if(event.type == sf::Event::Closed){
				win.close();
			}
		}
		win.clear();
		gtd.renderer.draw(&win);
		win.display();
		
	}
	
	std::cout << "App Closed" << std::endl;
	return 0;
}