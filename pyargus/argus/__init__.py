from argus import _argus
from argus._argus import *

__doc__ = _argus.__doc__
if hasattr(_argus, "__all__"):
    __all__ = _argus.__all__