#include <stdio.h>
#include "../include/red_black_tree_dict.h"

int main() {
    printf("--- C Dictionary Demo ---\n");

    // 1. Create a new dictionary
    printf("\n1. Creating a new dictionary.\n");
    Dictionary *dict = dict_new();
    if (!dict) {
        printf("Failed to create dictionary.\n");
        return 1;
    }

    // 2. Insert elements
    printf("\n2. Inserting key-value pairs:\n");
    printf("   - Inserting (10, 'ten')\n");
    dict_insert(dict, 10, "ten");
    printf("   - Inserting (20, 'twenty')\n");
    dict_insert(dict, 20, "twenty");
    printf("   - Inserting (5, 'five')\n");
    dict_insert(dict, 5, "five");

    // 3. Check for keys
    printf("\n3. Checking for keys:\n");
    printf("   - Contains key 10? %s\n", dict_contains_key(dict, 10) ? "true" : "false");
    printf("   - Contains key 15? %s\n", dict_contains_key(dict, 15) ? "true" : "false");

    // 4. Get values
    printf("\n4. Getting values:\n");
    const char *val = dict_get(dict, 10);
    if (val) {
        printf("   - Value for key 10: %s\n", val);
    }
    val = dict_get(dict, 15);
    if (!val) {
        printf("   - Value for key 15: Not found (as expected)\n");
    }

    // 5. Remove an element
    printf("\n5. Removing an element:\n");
    printf("   - Removing key 20...\n");
    dict_remove(dict, 20);
    printf("   - Contains key 20 after removal? %s\n", dict_contains_key(dict, 20) ? "true" : "false");
    printf("   - Contains key 10 after removal? %s\n", dict_contains_key(dict, 10) ? "true" : "false");

    // 6. Free the dictionary
    printf("\n6. Freeing the dictionary.\n");
    dict_free(dict);

    printf("\n--- Demo Complete ---\n");
    return 0;
}
