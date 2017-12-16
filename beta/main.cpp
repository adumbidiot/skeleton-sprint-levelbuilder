//#define SFML_STATIC
#include <SFML/Window.hpp>
#include <iostream>
#include "logicLayer/logicLayer.h"

int main(){
	sf::Window win(sf::VideoMode(1920, 1080), "Skeleton Sprint Levelbuilder");
	LogicLayer logic();
	
	while(win.isOpen()){
		sf::Event event;
		while(win.pollEvent(event)){
			if(event.type == sf::Event::Closed){
				win.close();
			}
		}
	}
	std::cout << "App Closed" << std::endl;
	return 0;
}