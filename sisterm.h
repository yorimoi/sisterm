//For serial
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <termios.h>

//For telnet
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include <stdio.h>
#include <time.h>
#include <regex.h>


// Keyboard Hit Check
int kbhit();

// All regcomp
int regcompAll();

// Check syntax
int syntaxCheck(unsigned char *str);

// Syntax OK -> repaint
void repaint(unsigned char *color);

// Control coloring
void coloring(unsigned char c);

// Warning with no argments
void nothingArgs(char *argv0, char op);

// Show version
// ( major.minor.tweak )
void version();

// Show usage
void usage(char *v);