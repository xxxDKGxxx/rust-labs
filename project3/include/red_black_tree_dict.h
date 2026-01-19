#ifndef RED_BLACK_TREE_DICT_H
#define RED_BLACK_TREE_DICT_H

#include <stdint.h>
#include <stdbool.h>

// Opaque pointer to the Dictionary struct.
typedef struct Dictionary Dictionary;

// Creates a new dictionary.
// Returns a pointer to the new dictionary, or NULL if allocation fails.
// The caller is responsible for freeing the dictionary with dict_free.
Dictionary *dict_new(void);

// Frees all memory associated with the dictionary.
void dict_free(Dictionary *dict);

// Inserts a key-value pair into the dictionary.
// If the key already exists, the value is updated.
void dict_insert(Dictionary *dict, uint64_t key, const char *value);

// Retrieves the value associated with a key.
// Returns a pointer to the value string, or NULL if the key is not found.
// The returned string is owned by the dictionary and should not be freed by the caller.
// It remains valid until the next mutable operation on the dictionary.
const char *dict_get(const Dictionary *dict, uint64_t key);

// Checks if the dictionary contains a key.
// Returns true if the key exists, false otherwise.
bool dict_contains_key(const Dictionary *dict, uint64_t key);

// Removes a key-value pair from the dictionary.
void dict_remove(Dictionary *dict, uint64_t key);

#endif // RED_BLACK_TREE_DICT_H
