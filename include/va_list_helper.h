#include <stdarg.h>
/**
 * @brief The callback function which is used as the continuation to call the va_list function
 * @param data The callback additoinal data
 * @param ap The valist pointer
 * @return nothing
 **/
typedef void (*rust_va_list_callback_func_t)(va_list ap, void* data);

/**
 * @brief The wrapper function for rust calling a function with valist pointer
 * @param cont The continuation function
 * @param data The additional data pointer
 * @return nothing
 **/
typedef void (*rust_va_list_wrapper_func_t)(rust_va_list_callback_func_t cont, void* data, ...);
