#------------------------------------------------------------------------------
SOURCE=./src/main.cpp
MYPROGRAM=primerug
LIBS=-lgmp
CC=g++

#------------------------------------------------------------------------------

all: $(MYPROGRAM)

$(MYPROGRAM): $(SOURCE)
	$(CC) -O3 $(SOURCE) -o $(MYPROGRAM) $(LIBS)

clean:
	rm -f $(MYPROGRAM) tuples.txt
