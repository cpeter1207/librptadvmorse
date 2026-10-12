/** @file descriptor_smoke.c @brief Exercise the installed dynamic Morse ABI. */
#include <assert.h>
#include <stdint.h>
#include <string.h>

#include "rptadvmorse/rptadvmorse.h"

static uint32_t bits(float value)
{
	uint32_t result;
	memcpy(&result, &value, sizeof(result));
	return result;
}

int main(void)
{
	const struct rptadv_morse_descriptor *api = rptadv_morse_descriptor();
	struct rptadv_morse_renderer *renderer = NULL;
	const uint8_t text[] = "E T I E";
	float first[960] = { 0 };
	float replay[960] = { 0 };
	size_t written = 0;

	assert(api != NULL);
	assert(api->struct_size == sizeof(*api));
	assert(api->abi_version == RPTADV_MORSE_ABI_VERSION);
	assert(strcmp(api->capability_name, "rptadv.morse") == 0);
	assert(api->create && api->render && api->restart && api->destroy);
	assert(api->create(text, sizeof(text) - 1, 37, 1250.0F, -11,
			   &renderer) == RPTADV_MORSE_OK);
	assert(renderer != NULL);
	assert(api->render(renderer, first, 960, &written) == RPTADV_MORSE_OK);
	assert(written == 960);
	assert(bits(first[0]) == 0);
	assert(bits(first[1]) == 0x3d3c0c51U);
	assert(api->restart(renderer) == RPTADV_MORSE_OK);
	assert(api->render(renderer, replay, 960, &written) == RPTADV_MORSE_OK);
	assert(written == 960);
	for (size_t index = 0; index < 960; ++index)
		assert(bits(replay[index]) == bits(first[index]));
	assert(api->render(renderer, NULL, 0, &written) == RPTADV_MORSE_OK);
	assert(written == 0);
	api->destroy(renderer);
	assert(api->create((const uint8_t *)"*", 1, 20, 1000.0F, -6,
			   &renderer) == RPTADV_MORSE_INVALID_ARGUMENT);
	assert(renderer == NULL);
	api->destroy(NULL);
	return 0;
}
