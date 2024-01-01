import os,sys,math,scipy,imp
import numpy as np
import scipy.io as scio
import torch, torchvision
import torch.nn as nn
from scipy import signal
from math import sqrt,log,pi
from torch.fft import fft,ifft
from torch.nn.functional import relu, softmax, cross_entropy
from torch import sigmoid,tanh
from torch.nn import MSELoss as MSE


# Definition
wind_len = 125                  # <====
wind_type = 'gaussian'          # <====
n_max_freq_component = 3
AWGN_amp = 0.01
str_modelname_prefix = './SEN_Results/SEN_' + wind_type + '_W' + str(wind_len)
str_model_name_pretrained = str_modelname_prefix + '_E1000.pt'      # <====
feature_len = 121
padded_len = 1000
crop_len = feature_len
blur_matrix_left = []

# Hyperparameters
n_begin_epoch = 1       # <====
n_epoch = 10000          # <====
n_itr_per_epoch = 500     # <====
n_batch_size = 64       # <====
n_test_size = 200
f_learning_rate = 0.001

class m_Linear(nn.Module):
    def __init__(self, size_in, size_out):
        super().__init__()
        self.size_in, self.size_out = size_in, size_out

        # Creation
        self.weights_real = nn.Parameter(torch.randn(size_in, size_out, dtype=torch.float32))
        self.weights_imag = nn.Parameter(torch.randn(size_in, size_out, dtype=torch.float32))
        self.bias = nn.Parameter(torch.randn(2, size_out, dtype=torch.float32))

        # Initialization
        nn.init.xavier_uniform_(self.weights_real, gain=1)
        nn.init.xavier_uniform_(self.weights_imag, gain=1)
        nn.init.zeros_(self.bias)
    
    def swap_real_imag(self, x):
        # [@,*,2,Hout]
        # [real, imag] => [-1*imag, real]
        h = x                   # [@,*,2,Hout]
        h = h.flip(dims=[-2])   # [@,*,2,Hout]  [real, imag]=>[imag, real]
        h = h.transpose(-2,-1)  # [@,*,Hout,2]
        h = h * torch.tensor([-1,1]).cuda()     # [@,*,Hout,2] [imag, real]=>[-1*imag, real]
        h = h.transpose(-2,-1)  # [@,*,2,Hout]
        
        return h

    def forward(self, x):
        # x: [@,*,2,Hin]
        h = x           # [@,*,2,Hin]
        h1 = torch.matmul(h, self.weights_real) # [@,*,2,Hout]
        h2 = torch.matmul(h, self.weights_imag) # [@,*,2,Hout]
        h2 = self.swap_real_imag(h2)            # [@,*,2,Hout]
        h = h1 + h2                             # [@,*,2,Hout]
        h = torch.add(h, self.bias)             # [@,*,2,Hout]+[2,Hout]=>[@,*,2,Hout]
        return h

def complex_array_to_bichannel_float_tensor(x):
    # x: (ndarray.complex128) [@,*,H]
    # ret: (tensor.float32) [@,*,2,H]
    x = x.astype('complex64')
    x_real = x.real     # [@,*,H]
    x_imag = x.imag     # [@,*,H]
    ret = np.stack((x_real,x_imag), axis=-2)    # [@,*,H]=>[@,*,2,H]
    ret = torch.tensor(ret)
    return ret

def bichannel_float_tensor_to_complex_array(x):
    # x: (tensor.float32) [@,*,2,H]
    # ret: (ndarray.complex64) [@,*,H]
    x = x.numpy()
    x = np.moveaxis(x,-2,0)  # [@,*,2,H]=>[2,@,*,H]
    x_real = x[0,:]
    x_imag = x[1,:]
    ret = x_real + 1j*x_imag
    return ret

def generate_blur_matrix_complex(wind_type, wind_len=251, padded_len=1000, crop_len=121):
    # Generate matrix used to introduce spec leakage in complex domain
    # ret: (ndarray.complex128) [crop_len, crop_len](row first)
    # Row first: each row represents the spectrum of one single carrier

    # Steps: carrier/windowing/pad/fft/crop/unwrap/norm

    # Parameters offloading
    fs = 1000
    n_f_bins = crop_len
    f_high = int(n_f_bins/2)
    f_low = -1 * f_high
    init_phase = 0

    # Carrier
    t_ = np.arange(0,wind_len).reshape(1,wind_len)/fs       # [1,wind_len] (0~wind_len/fs seconds)
    freq = np.arange(f_low,f_high+1,1).reshape(n_f_bins,1)  # [n_f_bins,1] (f_low~f_high Hz)
    phase = 2 * pi * freq * t_ + init_phase                 # [n_f_bins,wind_len]
    signal = np.exp(1j*phase)                               # [n_f_bins,wind_len]~[121,251]

    # Windowing
    if wind_type == 'gaussian':
        window = scipy.signal.windows.gaussian(wind_len, (wind_len-1)/sqrt(8*log(200)), sym=True)   # [wind_len,]
    else:
        window = scipy.signal.get_window(wind_type, wind_len)
    sig_wind = signal * window       # [n_f_bins,wind_len]*[wind_len,]=[n_f_bins,wind_len]~[121,251]

    # Pad/FFT
    sig_wind_pad = np.concatenate((sig_wind, np.zeros((n_f_bins,padded_len-wind_len))),axis=1)  # [n_f_bins,wind_len]=>[n_f_bins,padded_len]
    sig_wind_pad_fft = np.fft.fft(sig_wind_pad, axis=-1)    # [n_f_bins,padded_len]~[121,1000]

    # Crop
    n_freq_pos = f_high + 1
    n_freq_neg = abs(f_low)
    sig_wind_pad_fft_crop = np.concatenate((sig_wind_pad_fft[:,:n_freq_pos],\
        sig_wind_pad_fft[:,-1*n_freq_neg:]), axis=1)      # [n_f_bins,crop_len]~[121,121]

    # Unwrap
    n_shift = n_freq_neg
    sig_wind_pad_fft_crop_unwrap = np.roll(sig_wind_pad_fft_crop, shift=n_shift, axis=1) # [n_f_bins,crop_len]~[121,121]

    # Norm (amp_max=1)
    _sig_amp = np.abs(sig_wind_pad_fft_crop_unwrap)
    _sig_ang = np.angle(sig_wind_pad_fft_crop_unwrap)
    _max = np.tile(_sig_amp.max(axis=1,keepdims=True), (1,crop_len))
    _min = np.tile(_sig_amp.min(axis=1,keepdims=True), (1,crop_len))
    _sig_amp_norm = _sig_amp / _max
    sig_wind_pad_fft_crop_unwrap_norm = _sig_amp_norm * np.exp(1j*_sig_ang)

    # Return
    ret = sig_wind_pad_fft_crop_unwrap_norm

    return ret

def syn_one_batch_complex(blur_matrix_right, feature_len, n_batch):
    # Syn. HiFi, blurred and AWGN signal in complex domain
    # ret: (ndarray.complex128) [@,feature_len]
    # blur_matrix_right: Row first (each row represents the spectrum of one single carrier)

    # Syn. x [@,feature_len]
    x = np.zeros((n_batch, feature_len))*np.exp(1j*0)
    for i in range(n_batch):
        num_carrier = int(np.random.randint(0,n_max_freq_component,1))
        idx_carrier = np.random.permutation(feature_len)[:num_carrier]
        x[i,idx_carrier] = np.random.rand(1,num_carrier) * np.exp(1j*( 2*pi*np.random.rand(1,num_carrier) - pi ))

    # Syn. x_blur [@,feature_len]
    x_blur = x @ blur_matrix_right

    # Syn. x_tilde [@,feature_len]
    x_tilde = x_blur + 2*AWGN_amp*(np.random.random(x_blur.shape)-0.5) *\
        np.exp(1j*( 2*pi*np.random.random(x_blur.shape) - pi ))

    return x, x_blur, x_tilde

def loss_function(x, y):
    # x,y: [@,*,2,H]
    x = torch.linalg.norm(x,dim=-2) # [@,*,2,H]=>[@,*,H]
    y = torch.linalg.norm(y,dim=-2) # [@,*,2,H]=>[@,*,H]

    # MSE loss for Amp
    loss_recon = MSE(reduction='mean')(x, y)
    return loss_recon

class SEN(nn.Module):
    def __init__(self, feature_len):
        super(SEN, self).__init__()
        self.feature_len = feature_len

        # MLP for Regression
        self.fc_1 = m_Linear(feature_len, feature_len)
        self.fc_2 = m_Linear(feature_len, feature_len)
        self.fc_3 = m_Linear(feature_len, feature_len)
        self.fc_4 = m_Linear(feature_len, feature_len)
        self.fc_out = m_Linear(feature_len, feature_len)

    def forward(self, x):
        h = x   # (@,*,2,H)

        h = tanh(self.fc_1(h))          # (@,*,2,H)=>(@,*,2,H)
        h = tanh(self.fc_2(h))          # (@,*,2,H)=>(@,*,2,H)
        h = tanh(self.fc_3(h))          # (@,*,2,H)=>(@,*,2,H)
        h = tanh(self.fc_4(h))          # (@,*,2,H)=>(@,*,2,H)
        output = tanh(self.fc_out(h))   # (@,*,2,H)=>(@,*,2,H)

        return output

def train(model, blur_matrix_right, feature_len, n_epoch, n_itr_per_epoch, n_batch_size, optimizer):
    for i_epoch in range(n_begin_epoch, n_epoch+1):
        model.train()
        total_loss_this_epoch = 0
        for i_itr in range(n_itr_per_epoch):
            x, _, x_tilde = syn_one_batch_complex(blur_matrix_right=blur_matrix_right, feature_len=feature_len, n_batch=n_batch_size)
            x = complex_array_to_bichannel_float_tensor(x)
            x_tilde = complex_array_to_bichannel_float_tensor(x_tilde)
            x = x.cuda()
            x_tilde = x_tilde.cuda()

            optimizer.zero_grad()
            y = model(x_tilde)
            loss = loss_function(x, y)
            loss.backward()
            optimizer.step()
            
            total_loss_this_epoch += loss.item()
            
            if i_itr % 10 == 0:
                print('--------> Epoch: {}/{} loss: {:.4f} [itr: {}/{}]'.format(
                    i_epoch+1, n_epoch, loss.item() / n_batch_size, i_itr+1, n_itr_per_epoch), end='\r')
        
        # Validate
        model.eval()
        x, _, x_tilde = syn_one_batch_complex(blur_matrix_right=blur_matrix_right, feature_len=feature_len, n_batch=n_batch_size)
        x = complex_array_to_bichannel_float_tensor(x)
        x_tilde = complex_array_to_bichannel_float_tensor(x_tilde)
        x = x.cuda()
        x_tilde = x_tilde.cuda()
        y = model(x_tilde)
        total_valid_loss = loss_function(x, y)
        print('========> Epoch: {}/{} Loss: {:.4f}'.format(i_epoch+1, n_epoch, total_valid_loss) + ' ' + wind_type + '_' + str(wind_len) + ' '*20)

        if i_epoch % 500 == 0:
            torch.save(model, str_modelname_prefix+'_E'+str(i_epoch)+'.pt')

# ======================== Start Here ========================
if __name__ == "__main__":
    if len(sys.argv) < 1:
        print('Please specify which GPU to use ...')
        exit(0)
    if (sys.argv[1] == '1' or sys.argv[1] == '0'):
        os.environ["CUDA_VISIBLE_DEVICES"] = sys.argv[1]
    else:
        print('Wrong GPU number, 0 or 1 supported!')
        exit(0)

    # Generate blur matrix
    blur_matrix_right = generate_blur_matrix_complex(wind_type=wind_type, wind_len=wind_len, padded_len=padded_len, crop_len=crop_len)

    # Load or fabricate model
    print('Model building...')
    model = SEN(feature_len=feature_len)
    model.cuda()

    # Train model
    print('Model training...')
    train(model=model, blur_matrix_right=blur_matrix_right, feature_len=feature_len, n_epoch=n_epoch, n_itr_per_epoch=n_itr_per_epoch, n_batch_size=n_batch_size, optimizer=torch.optim.RMSprop(model.parameters(), lr=f_learning_rate))
