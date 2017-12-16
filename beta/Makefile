EXECUTABLE_NAME = skeleton-sprint-levelbuilder
EXECUTABLE_NAME_WIN = $(EXECUTABLE_NAME).exe

INCLUDE_DIR_SFML_WIN_FLAG = -Iwin/SFML-2.4.2/include

SFML_LIBS = -lsfml-graphics-s -lsfml-window -lsfml-system
SFML_LIBRARY_DIR_WIN_FLAG = -Lwin/SFML-2.4.2/lib

OBJ_FILES_ALL = main.o logicLayer.o
CXX = g++
	
.PHONY: win test

test: win
	$(EXECUTABLE_NAME)
win: $(SRC_FILES_ALL) logicLayer.o main.o
	$(CXX) main.o logicLayer.o $(SFML_LIBRARY_DIR_WIN_FLAG) $(SFML_LIBS) -mwindows -o $(EXECUTABLE_NAME_WIN) 

main.o: main.cpp
	$(CXX) -c main.cpp -o main.o $(INCLUDE_DIR_SFML_WIN_FLAG) 
logicLayer.o: logicLayer/logicLayer.cpp
	$(CXX) -c logicLayer/logicLayer.cpp -o logicLayer.o