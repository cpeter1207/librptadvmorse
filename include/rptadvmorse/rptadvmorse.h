/**
 * @file rptadvmorse.h
 * @brief ABI 1 for 48 kHz Morse rendering through the tone oscillator.
 */
#ifndef RPTADVMORSE_H
#define RPTADVMORSE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** ABI major carried by the descriptor and SONAME. */
#define RPTADV_MORSE_ABI_VERSION 1U
/** Successful operation. */
#define RPTADV_MORSE_OK 0
/** Invalid pointer, UTF-8, Morse character, or configuration. */
#define RPTADV_MORSE_INVALID_ARGUMENT 1

/** Opaque, setup-allocated Morse renderer. */
struct rptadv_morse_renderer;

/** Immutable ABI 1 entrypoint table. Resolve and validate during setup. */
struct rptadv_morse_descriptor {
	/** Size of this structure in bytes. */
	uint32_t struct_size;
	/** Major ABI version. */
	uint32_t abi_version;
	/** NUL-terminated identity, "rptadv.morse". */
	const char *capability_name;
	/** Validate UTF-8 message and allocate its segment schedule and oscillator. */
	int32_t (*create)(const uint8_t *text, size_t text_len,
			  uint32_t speed_wpm, float frequency_hz, int8_t level_db,
			  struct rptadv_morse_renderer **out);
	/** Render the next samples; reports the count written to output. */
	int32_t (*render)(struct rptadv_morse_renderer *renderer,
			  float *output, size_t capacity, size_t *written);
	/** Replay from the first element with its initial phase. */
	int32_t (*restart)(struct rptadv_morse_renderer *renderer);
	/** Release a renderer after callback users stop; NULL is accepted. */
	void (*destroy)(struct rptadv_morse_renderer *renderer);
};

/** Return the immutable descriptor; it remains valid for the library lifetime. */
const struct rptadv_morse_descriptor *rptadv_morse_descriptor(void);

#ifdef __cplusplus
}
#endif
#endif
