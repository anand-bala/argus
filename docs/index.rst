Argus: Temporal Logic Monitoring Tool
=====================================

Argus aims to be a tool to generate monitors for Signal Temporal Logic (STL), and its
different semantics.

This library is a direct successor of my `signal-temporal-logic
<https://github.com/anand-bala/signal-temporal-logic/>`_ tool, and is inspired by the
following projects:

- `py-metric-temporal-logic`_ is a tool written in pure Python, and provides an elegant
  interface for evaluating discrete time signals using Metric Temporal Logic (MTL).
- `RTAMT`_ is a Python library for offline and online
  monitoring of STL specifications.
- `Breach`_ and `S-TaLiRo`_ are Matlab toolboxes designed for falsification and
  simulation-based testing of cyber-physical systems with STL and MTL specifications,
  respectively. One of their various features includes the ability to evaluate the
  robustness of signals against STL/MTL specifications.

The goal of this tool is to provide offline and online monitors for Signal Temporal
Logic (STL) and its semantics, focussing on performance and ease of use in controllers
synthesis and analysis (for training and monitoring them).

The project name is inspired from `Argus Panoptes`_.

.. _Argus Panoptes: https://www.britannica.com/topic/Argus-Greek-mythology
.. _py-metric-temporal-logic: https://github.com/mvcisback/py-metric-temporal-logic/
.. _Breach: https://github.com/decyphir/breach
.. _S-TaLiRo: https://sites.google.com/a/asu.edu/s-taliro/s-taliro
.. _RTAMT: https://github.com/nickovic/rtamt

Installing
----------

Currently, I am not publishing the library to PyPI, so you will have to install the
package directly from the wheel files in the `latest Github release
<https://github.com/anand-bala/argus/releases/latest>`_. For example, to install version
``0.1.0`` of the library for Python 3.10 on a Linux distribution running on a 64-bit
Intel/AMD machine, you just need to do:

.. code-block:: bash

   pip install https://github.com/anand-bala/argus/releases/download/v0.1.0/pyargus-0.1.0-cp310-cp310-manylinux_2_17_x86_64.manylinux2014_x86_64.whl


Contents
--------

.. toctree::
   :maxdepth: 2

   getting_started


Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
