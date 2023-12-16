#include <stdint.h>

//111111 - exponent of 
const uint32_t e_mask = (1<<6)-1;
//111111111111
const uint32_t mantissa_mask = (1<<12)-1;
//1
const uint32_t sign_mask = 1;
//111111111111000000
const uint32_t r_mant_mask = (((1<<11) - 1) << 18);
//same for imag
const uint32_t i_mant_mask = (((1<<11) - 1) << 6);
//
const uint32_t r_sign_mask = (1<<29);
const uint32_t i_sign_mask = (1<<17);

const uint32_t count_mask = (1<<10);
const uint32_t mant_mask = (1<<10)-1;

void wiros_parse_csi(const int n_sub, const uint32_t *csi, double *r_out, double *i_out) {
  //decode CSI
  uint64_t c_r, c_i;
  for(int i = 0; i < n_sub; ++i){

	uint32_t c = (uint64_t)csi[i];
	c_r=0;
	c_i=0;
	uint32_t exp = ((int32_t)(c & e_mask) - 31 + 1023);
	uint32_t r_exp = exp;
	uint32_t i_exp = exp;

	uint32_t r_mant = (c&r_mant_mask) >> 18;
	uint32_t i_mant = (c&i_mant_mask) >> 6;

	//construct real mantissa
	uint32_t e_shift = 0;
	while(!(r_mant & count_mask)){
	  r_mant *= 2;
	  e_shift += 1;
	  if(e_shift == 10){
		r_exp = 1023;
		e_shift = 0;
		r_mant = 0;
		break;
	  }
	}
	r_exp -= e_shift;

	//construct imaginary mantissa
	e_shift = 0;
	while(!(i_mant & count_mask)){
	  i_mant *= 2;
	  e_shift += 1;
	  if(e_shift == 10){
		i_exp = 1023;
		e_shift = 0;
		i_mant = 0;
		break;
	  }
	}
	i_exp -= e_shift;

	//construct doubles
	c_r |= (uint64_t)(c & r_sign_mask) << 34;
	c_i |= (uint64_t)(c & i_sign_mask) << 46;

	c_r |= ((uint64_t)(r_mant & mant_mask)) << 42;
	c_i |= ((uint64_t)(i_mant & mant_mask)) << 42;

	c_r |= ((uint64_t)r_exp)<<52;
	c_i |= ((uint64_t)i_exp)<<52;

	//place doubles
	r_out[i] = c_r;
	i_out[i] = c_i;

  }
}
