EXECUTABLE_NAME = generic-defense-dot-ee-ex-ee
EXECUTABLE_NAME_WIN = $(EXECUTABLE_NAME).exe

INCLUDE_DIR_SFML_WIN_FLAG = -Iwin/SFML-2.4.2/include

SFML_LIBS = -lsfml-graphics-s -lsfml-window -lsfml-system -lsfml-graphics
SFML_LIBRARY_DIR_WIN_FLAG = -Lwin/SFML-2.4.2/lib

OBJ_FILES_ALL = build/main.o build/logicLayer.o
CXX = g++
	
.PHONY: win test

test: win build/main.o build/logicLayer.o
	$(EXECUTABLE_NAME)
win: $(SRC_FILES_ALL) $(OBJ_FILES_ALL)
	$(CXX) build/main.o build/logicLayer.o $(SFML_LIBRARY_DIR_WIN_FLAG) $(SFML_LIBS) -mwindows -o $(EXECUTABLE_NAME_WIN) 
build/main.o: main.cpp include/logicLayer.h
	$(CXX) -c main.cpp -o build/main.o $(INCLUDE_DIR_SFML_WIN_FLAG) -Iinclude
build/logicLayer.o: logicLayer/logicLayer.cpp
	$(CXX) -c logicLayer/logicLayer.cpp -o build/logicLayer.o