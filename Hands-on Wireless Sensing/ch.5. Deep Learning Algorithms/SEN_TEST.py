import os,sys,math,torch,scipy
import numpy as np
import scipy.io as scio
import torch.nn as nn
from scipy import signal
from math import sqrt,log,pi
from torch.fft import fft,ifft
from torch.nn.functional import relu, softmax, cross_entropy
from torch import sigmoid
from torch.nn import MSELoss as MSE
from SEN_trainer import m_Linear, SEN, generate_blur_matrix_complex, complex_array_to_bichannel_float_tensor, bichannel_float_tensor_to_complex_array

W = 125
wind_type = 'gaussian'
str_model_name = './SEN_Results/SEN_' + wind_type + '_W' + str(W) + '_E2500.pt'
file_path_csi = '/srv/node/sdc1/zhangyi/Widar3_data/GES_CSI_MAT/20181130/hkh/hkh-1-3-2-1.mat'

def STFT(signal, fs=1, stride=1, wind_wid=5, dft_wid=5, window_type='gaussian'):
    assert dft_wid >= wind_wid and wind_wid > 0 and stride <= wind_wid and stride > 0\
        and isinstance(stride, int) and isinstance(wind_wid, int) and isinstance(dft_wid, int)\
        and isinstance(fs, int) and fs > 0

    if window_type == 'gaussian':
        window = scipy.signal.windows.gaussian(wind_wid, (wind_wid-1)/sqrt(8*log(200)), sym=True)
    elif window_type == 'rect':
        window = np.ones((wind_wid,))
    else:
        window = scipy.signal.get_window(window_type, wind_wid)
    
    f_bins, t_bins, stft_spectrum = scipy.signal.stft(x=signal, fs=fs, window=window, nperseg=wind_wid, noverlap=wind_wid-stride, nfft=dft_wid,\
        axis=-1, detrend=False, return_onesided=False, boundary='zeros', padded=True)
    
    return f_bins, stft_spectrum

def normalize_data(data_1):
    # max=1
    # data(ndarray.complex)=>data_norm(ndarray.complex): [6,121,T]=>[6,121,T]
    data_1_abs = abs(data_1)
    data_1_max = data_1_abs.max(axis=(1,2),keepdims=True)     # [6,121,T]=>[6,1,1]
    data_1_max_rep = np.tile(data_1_max,(1,data_1_abs.shape[1],data_1_abs.shape[2]))    # [6,1,1]=>[6,121,T]
    data_1_norm = data_1 / data_1_max_rep
    return  data_1_norm

def csi_to_spec():
    global file_path_csi
    global W
    signal = scio.loadmat(file_path_csi)['csi_mat'].transpose() # [6,T] complex
    # STFT
    _, spec = STFT(signal, fs=1000, stride=1, wind_wid=W, dft_wid=1000, window_type='gaussian') # [6,1000,T]j
    # Crop
    spec_crop = np.concatenate((spec[:,:61], spec[:,-60:]), axis=1) # [1,1000,T]j=>[1,121,T]j
    # Unwrap
    spec_crop_unwrap = np.roll(spec_crop, shift=60, axis=1) # [1,121,T]j
    # Normalize
    spec_crop_unwrap_norm = normalize_data(spec_crop_unwrap)     # [6,121,T] complex
    if np.sum(np.isnan(spec_crop_unwrap_norm)):
        print('>>>>>>>>> NaN detected!')
    ret = spec_crop_unwrap_norm
    return ret

def syn_spec():
    # Syn. spectrum with STFT
    fs = 1000
    N_sample = 4*fs     # 0 ~ N_sample/fs seconds
    init_phase = 0

    # Syn. signal
    # ========Method-1
    # t_ = np.arange(0,N_sample).reshape(1,N_sample)/fs       # [1,N_sample]
    # freq1 = 10
    # freq2 = 20
    # phase1 = 2 * pi * freq1 * t_        # [1,N_sample]
    # phase2 = 2 * pi * freq2 * t_        # [1,N_sample]
    
    # ========Method-2
    delta_t = 1/fs
    freq_diff_1 = np.linspace(1/4,1/1,N_sample)
    freq_diff_2 = np.linspace(1/4,3/4,N_sample)
    freq1 = 30 * np.sin(2 * pi * np.cumsum(freq_diff_1) * delta_t)
    freq2 = 30 * np.sin(2 * pi * np.cumsum(freq_diff_2) * delta_t)
    # freq1 = 30 * np.sin(2 * pi * (1/4) * np.arange(0, N_sample) / fs)
    # freq2 = 40 * np.sin(2 * pi * (1/3) * np.arange(0, N_sample) / fs)
    freq3 = freq2 - 8
    freq4 = freq3 - 10
    freq5 = freq4 - 10
    # ========
    phase1 = 2 * pi * np.cumsum(freq1) * delta_t
    phase2 = 2 * pi * np.cumsum(freq2) * delta_t
    phase3 = 2 * pi * np.cumsum(freq3) * delta_t
    phase4 = 2 * pi * np.cumsum(freq4) * delta_t
    phase5 = 2 * pi * np.cumsum(freq5) * delta_t
    # ========
    signal1 = 2*np.exp(1j*phase1)
    signal2 = 2*np.exp(1j*phase2)
    signal3 = 2*np.exp(1j*phase3)
    signal4 = np.exp(1j*phase4)
    signal5 = 2*np.exp(1j*phase5)
    # ========
    signal = signal1 + signal2
    signal = np.reshape(signal, (1,-1))
    _, spec = STFT(signal, fs=fs, stride=1, wind_wid=W, dft_wid=1000, window_type='gaussian') # [1,1000,T]j
    # Crop
    spec_crop = np.concatenate((spec[:,:61], spec[:,-60:]), axis=1) # [1,1000,T]j=>[1,121,T]j
    # Unwrap
    spec_crop_unwrap = np.roll(spec_crop, shift=60, axis=1) # [1,121,T]j
    # Norm (max=1 norm for amp)
    spec_crop_unwrap_norm = normalize_data(spec_crop_unwrap)
    return spec_crop_unwrap_norm

# ======================== Start Here ========================
if __name__ == "__main__":
    # Load trained model
    print('Loading model...')
    model = torch.load(str_model_name)

    print('Testing model...')
    model.eval()
    with torch.no_grad():
        # Import raw spectrogram
        data_1 = csi_to_spec()
        
        # Enhance spectrogram
        x_tilde = complex_array_to_bichannel_float_tensor(data_1)   # [6,121,T]=>[6,121,2,T]
        x_tilde = x_tilde.permute(0,3,2,1)              # [6,121,2,T]=>[6,T,2,121]
        y = model(x_tilde.cuda()).cpu()                 # [6,T,2,121]
        y = bichannel_float_tensor_to_complex_array(y)  # [6,T,121]
        y = np.transpose(y,(0,2,1))                     # [6,T,121]=>[6,121,T]
        scio.savemat('SEN_test_x_tilde_complex_W' + str(W) + '.mat', {'x_tilde':data_1})
        scio.savemat('SEN_test_y_complex_W' + str(W) + '.mat', {'y':y})