Argus: Temporal Logic Monitoring Tool
=====================================

[![PyPI version](https://badge.fury.io/py/argus-temporal-logic.svg)](https://badge.fury.io/py/argus-temporal-logic)

[![codecov](https://codecov.io/gh/anand-bala/argus/graph/badge.svg?token=O2YXQPWTNS)](https://codecov.io/gh/anand-bala/argus)

Argus aims to be a tool to generate monitors for Signal Temporal Logic (STL), and its
different semantics.

This library is a direct successor of my
[`signal-temporal-logic`](https://github.com/anand-bala/signal-temporal-logic/) tool,
and is inspired by the following projects:

- [py-metric-temporal-logic] is a tool written in pure Python, and provides an elegant
  interface for evaluating discrete time signals using Metric Temporal Logic (MTL).
- [RTAMT] is a Python library for offline and online
  monitoring of STL specifications.
- [Breach] and [S-TaLiRo] are [Matlab] toolboxes designed for falsification and
  simulation-based testing of cyber-physical systems with STL and MTL specifications,
  respectively. One of their various features includes the ability to evaluate the
  robustness of signals against STL/MTL specifications.

The goal of this tool is to provide offline and online monitors for Signal Temporal
Logic (STL) and its semantics, focussing on performance and ease of use in controllers
(for training and monitoring them).

The project name is inspired from [Argus Panoptes].

[Argus Panoptes]: https://www.britannica.com/topic/Argus-Greek-mythology
[py-metric-temporal-logic]: https://github.com/mvcisback/py-metric-temporal-logic/
[Matlab]: https://www.mathworks.com/products/matlab.html
[Breach]: https://github.com/decyphir/breach
[S-TaLiRo]: https://sites.google.com/a/asu.edu/s-taliro/s-taliro
[RTAMT]: https://github.com/nickovic/rtamt
