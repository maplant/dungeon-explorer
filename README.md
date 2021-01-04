# Simple tile-based dungeon/city builder written in Rust

This is an implementation of an algorithm I wrote to create random maps for 
spelunky-like game I was creating in college. Although the game no longer exists,
the algorithm is a good demonstration of how to use various other algorithms.

Basically, the algorithm uses a depth-first search to align rectangular rooms
such that they touch each other along "entrances"/"exits", and uses a KD-tree to
quickly determine if they overlap.

The visual aspect of this program is pretty haphazardly put together and may 
not work correctly depending on your system. So much for using SDL2.
