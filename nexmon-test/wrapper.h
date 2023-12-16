#include <stdint.h>

void unpack_float_acphy(int nbits, int autoscale, int shft, int nman, int nexp, int nfft, const uint32_t *H, int32_t *Hout);
