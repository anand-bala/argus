Getting Started
===============

The only import required for using ``argus`` is the following:

.. code-block:: python

   import argus

While the core of the library is written in Rust, the ``argus`` package exports all the
features in the core library into the top level namespace.


Writing specifications
----------------------

There are two ways to write specifications in ``argus``:

1. Using the Python API.
2. Using a string-based parser.

For using the Python API, you have access to all the subclasses of :py:class:`argus.Expr`.

.. code-block:: python

   x = argus.VarFloat("x")
   y = argus.VarFloat("y")
   print(x > y)
   # Cmp(Cmp { op: Greater { strict: true }, lhs: FloatVar(FloatVar { name: "x" }), rhs: FloatVar(FloatVar { name: "y" }) })

Similarly, you can create arithmetic expressions using the builtin Python operators
(``+,-,*,/``) on numeric expressions, and Boolean expressions using the builtin Python
operators (``&,|,~``). Moreover, you can build temporal expressions as follows:

.. code-block:: python

   phi1 = argus.Eventually(x > y)
   phi2 = argus.Always(y < argus.ConstFloat(10.0), interval=(0, 10))


.. note::

   In the above code block, the argument for ``interval`` implies inclusive bounds
   always. While Signal Temporal Logic supports exclusive bounds, with real-valued
   signals it is practically impossible to exclude the boundaries of an interval.


On the otherhand, the string-based API uses :py:func:`argus.parse_expr` to generate the
expression directly.

.. code-block:: python

   phi1 = argus.parse_expr("F (x > y)")
   phi2 = argus.parse_expr("G[0, 10] (y < 10.0)")
   phi3 = argus.parse_expr("G[0..10] (y < 10.0)")
   # phi2 and phi3 are the same here, just different notation.


Creating signal traces
----------------------

To create signal traces using :py:class:`Trace`:

.. code-block:: python

   import random
   data = dict(
       x=argus.FloatSignal.from_samples([(i, random.random()) for i in range(20)]),
       y=argus.FloatSignal.constant(10.0),
   )
   trace = argus.Trace(data)

In the above, ``x`` and ``y`` are both signals that represent ``float``s. Here, ``x`` is
created using :py:func:`argus.FloatSignal.from_samples`, which creates a signal from
a list of 2-tuples containing the "timestamp" (``i`` here) and the sampled value
(``random.random()``).
Similarly, ``y`` is a constant signal, i.e., it has a constant value throughout it's
domain.


Monitoring traces
-----------------

To monitor traces, one can use either the builtin qualitative semantics (:py:func:`argus.eval_bool_semantics`) or quantitative semantics (:py:func:`argus.eval_robust_semantics`).

.. code-block:: python

   check: argus.BoolSignal = argus.eval_bool_semantics(phi1, trace)
   rob: argus.FloatSignal = argus.eval_robust_semantics(phi2, trace)

The above functions return signals (either Boolean or floating point), and the output of
the functions can be found at different points in the time domain where the
specifications are defined by using:

.. code-block:: python

   assert check.at(0) == False # x is never > y
   assert rob.at(0) == 0.0 # y is always == 10.0
