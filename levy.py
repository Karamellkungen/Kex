import numpy as np
from numpy.random import uniform
import matplotlib.pyplot as plt
import seaborn as sns
from scipy.stats import levy, norm

cutoff = 30

def inverse_transform(c = 1):
    return c / norm.ppf(uniform())**2

def sample_norm(c):
    return c / norm.rvs()**2

def scipy(c):
    return levy.rvs(scale = c)

def corrected(c, mu):
    sample = levy.rvs(scale = c, loc = mu)
    while sample > cutoff:
        sample = levy.rvs(scale = c, loc = mu)
    return sample

def rounded(c, mu):
    return round(corrected(c, mu))

def ceiled(c, mu):
    return np.ceil(corrected(c, mu))

def plot(c, mu, n, functions):
    #functions.append(scipy)
    for f in functions:
        #samples = np.array(list(filter(lambda x: x < 100, [f(c) for _ in range(n)])))
        samples = np.array([x if x < cutoff else cutoff for x in [f(c, mu) for _ in range(n)]])
        #sns.kdeplot(samples, cut=0, label=f.__name__)
        plt.hist(samples, bins=cutoff, density=True, histtype='step', label=f.__name__)
    plt.legend()
    plt.show()

plot(1.5, 1, 10000, [rounded, ceiled])