#include <stdint.h>

void unpack_float_acphy(int nbits, int autoscale, int shft, int nman, int nexp, int nfft, const uint32_t *H, int32_t *Hout);

void wiros_parse_csi(const int n_sub, const uint32_t *csi, double *r_out, double *i_out);
