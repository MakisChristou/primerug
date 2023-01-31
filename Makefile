#------------------------------------------------------------------------------
SOURCE=main.cpp
MYPROGRAM=primerug
LIBS=-lgmp
CC=g++

#------------------------------------------------------------------------------

all: main


main: main.cpp Config.h Stats.h Tools.h
	$(CC) -O3 -o $(MYPROGRAM) $^ $(LIBS)

clean:
	rm -f $(MYPROGRAM) *.o tuples.txt *.gch
