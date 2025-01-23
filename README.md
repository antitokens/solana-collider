# Antitoken Collider: A Prediction Framework for Continuous Outcomes

`sshmatrix` | `antitoken` | `q/acc`

## Abstract

The Antitoken Collider Protocol introduces a quantum-inspired tokenomics framework designed to advance decentralised market-making and decision-making systems. By utilising a pair of entangled tokens, `$ANTI` and `$PRO`, the protocol incorporates the Collider contract, which transforms these inputs into emission (`$BARYON`) and radiation (`$PHOTON`) tokens. This innovative mechanism integrates deterministic and probabilistic behaviours, allowing markets to reflect both stable and uncertain dynamics. The protocol’s dual-token architecture, rooted in quantum-like operations, is positioned to address challenges in prediction markets, decentralised science (DeSci), and other domains requiring nuanced representations of dualities such as trust vs. uncertainty or risk vs. reward.

## 1. Introduction

Blockchain-based decentralised systems have transformed finance and governance, offering novel mechanisms for automated market-making and resource allocation. However, traditional continuous automated market makers (AMMs) often fall short in applications where dualities, uncertainty, or non-linear outcomes are inherent. For instance, prediction markets and DeSci initiatives require tokenomics models capable of encoding probabilistic outcomes and balancing deterministic stability with dynamic adaptability.

The Antitoken Collider Protocol introduces a groundbreaking approach to these challenges by leveraging a dual-token architecture of `$ANTI` and `$PRO` tokens, which interact within a bespoke Collider contract. Inspired by principles of quantum mechanics, the Collider utilises tunable operations to emit `$BARYON` and `$PHOTON` tokens, representing predictable (deterministic) and uncertain (probabilistic) market dynamics, respectively. These emitted tokens facilitate a range of applications, from incentivising accurate predictions in market ecosystems to supporting decentralised research funding and distributed resource sharing.

This yellow paper presents the theoretical underpinnings, mathematical models, and practical applications of the Antitoken Collider Protocol. By introducing structured uncertainty, entangled token interactions, and reversible operations, this framework reimagines decentralised markets and offers a robust foundation for innovation across a variety of domains. The following sections delve into the protocol's design principles, operational mechanics, and potential use cases, setting the stage for broader adoption and adaptation in decentralised systems.

## 2. Core Mechanics

The protocol operates on a dual-token system where participants can deposit two types of tokens, `$ANTI` and `$PRO` , represented as `𝛂` and `𝞫` respectively. For any given market, the protocol calculates two fundamental values:

1. The `$BARYON` value (`μ`):

<pre>
μ = N<sub>BARYON</sub> = 0, if N<sub>ANTI</sub> + N<sub>PRO</sub> = 𝛂 + 𝞫 < 1

μ = N<sub>BARYON</sub> = |N<sub>ANTI</sub> - N<sub>PRO</sub>| = |𝛂 - 𝞫| otherwise
</pre>

i.e. 

<pre>
μ = 0, if 𝛂 + 𝞫 < 1

μ = |𝛂 - 𝞫| otherwise
</pre>

2. The `$PHOTON` value (`σ`):

<pre>
σ = N<sub>PHOTON</sub> = 0, if N<sub>ANTI</sub> + N<sub>PRO</sub> < 1 or |N<sub>ANTI</sub> - N<sub>PRO</sub>| = N<sub>ANTI</sub> + N<sub>PRO</sub>

σ = N<sub>PHOTON</sub> = N<sub>ANTI</sub> + N<sub>PRO</sub>, if 0 = N<sub>ANTI</sub> + N<sub>PRO</sub> < 1 or |N<sub>ANTI</sub> - N<sub>PRO</sub>| = N<sub>ANTI</sub> + N<sub>PRO</sub>

σ = N<sub>PHOTON</sub> = (N<sub>ANTI</sub> + N<sub>PRO</sub>)/|N<sub>ANTI</sub> - N<sub>PRO</sub>|, otherwise
</pre>

i.e.

<pre>
σ = 0, if 𝛂 + 𝞫 < 1 or |𝛂 - 𝞫| = 𝛂 + 𝞫

σ = 𝛂 + 𝞫, if 0 = 𝛂 + 𝞫 < 1 or |𝛂 - 𝞫| = 𝛂 + 𝞫

σ = (𝛂 + 𝞫)/|𝛂 - 𝞫|, otherwise
</pre>

In this formulation, `μ` captures the magnitude or size, while `σ` captures the confidence or certainty, of a user's prediction.

## 3. Closeness to Outcome

The overlap function `𝜪` plays central role in token redistribution following a prediction's finality. The overlap function is a measure of closeness of the prediction to any given truth. The overlap function is derived as follows:

<pre>
𝜪(𝞅<sub>u</sub>, 𝞅<sub>T</sub>) = <span style="font-size:22px;">⨏</span>𝞅<sub>u</sub>(𝛾).𝞅<sub>T</sub>(𝛾)d𝛾
</pre>

where, 𝞅<sub>u</sub> is a user's prediction and 𝞅<sub>T</sub> is the truth distribution; `⨏` represents a finite integral over the entire range of possible outcomes. Lastly, `𝜪 ∈ [0, 1]`.

### 2.1 Binary Outcomes

If the truth is binary (a strict `Yes` or `No` ), then 𝞅<sub>T</sub> becomes a dirac-delta function, i.e. 𝞅<sub>T</sub> = 𝞭(𝛾<sub>T</sub>). Consequently, the overlap function reduces to:

<pre>
𝜪(𝞅<sub>u</sub>, 𝞅<sub>T</sub>) = <span style="font-size:22px;">⨏</span>𝞅<sub>u</sub>(𝛾).𝞭(𝛾<sub>T</sub>)d𝛾 = 𝞅<sub>u</sub>(𝛾<sub>T</sub>).
</pre>

In explicit form, the overlap calculation for each position to the closest binary outcome (a `Yes` or `No` outcome) is defined as:

<pre>
𝜪(N<sub>BARYON</sub>, N<sub>PHOTON</sub>) = e<sup>-log<sub>10</sub>(S<sub>ANTI</sub> + S<sub>PRO</sub> - N<sub>BARYON</sub>)<sup>2</sup></sup>/2𝞻<sup>2</sup>(N<sub>PHOTON</sub>)
</pre>

i.e.

<pre>
𝜪(μ, σ) = e<sup>-log<sub>10</sub>(2.10<sup>9</sup> - μ)<sup>2</sup></sup>/2𝞻<sup>2</sup>(σ)
</pre>

where:

* <code>S<sub>ANTI</sub></code>, <code>S<sub>PRO</sub></code> are the total supplies of `$ANTI` and `$PRO` respectively, 
* <code>𝞻(σ) = 1 + log<sub>10</sub>(σ)</code> for `σ > 1`, and `1` otherwise, and
* `𝞅` are normal distributions. 

## 4. Token Redistribution

The token redistribution process based on the final outcome is called equalisation, using truth distribution with mean <code>T<sub>μ</sub></code> and standard deviation <code>T<sub>σ</sub></code>. The equalisation function utilises a binning mechanism using the values calculated by the overlap function <code>𝜪(𝞅<sub>u</sub>, 𝞅<sub>T</sub>)</code> for each user prediction <code>𝞅<sub>u</sub></code>. For a set of predictions, the entire range of overlap `𝜪` is binned into `N` bins. These bins, indexed by `i`, are then filled with the total tokens in the prediction pool, as some function <code>Τ(𝜪<sub>i</sub>)</code>; in the `alpha` version, this dependence is simply linear in `i`, i.e. <code>Τ(𝜪<sub>i</sub>) = i/N</code>. The overlap distribution is then given by:

<pre>
Γ<sub>IN</sub> = {Τ(𝜪<sub>1</sub>), ..., Τ(𝜪<sub>N</sub>)}.
</pre>

Once the bins are filled, each prediction is dropped into its corresponding bin based on its overlap value `𝜪(μ, σ)`. At the end of this process, each `i` bin now additionally contains <code>k<sub>i</sub></code> members among which <code>Τ(𝜪<sub>i</sub>)</code> tokens will be redistributed. If <code>k<sub>i</sub> = 0</code> or `1`, redistribution is trivial. If <code>k<sub>i</sub> > 1</code>, then tokens in that bin are redistributed in the same proportion as they were originally deposited by the user, i.e. <code>Η<sub>r</sub>/Η<sub>TOTAL</sub></code> for <code>r = {1, ..., k<sub>i</sub>}</code> where `Η` represents `$ANTI` and `$PRO`. At the end of this procedure, each user's deposit is rebalanced according to their closeness to the true outcome. The redistributed tokens indexed by `i` are then described by:

<pre>
Γ<sub>OUT</sub> = {{Γ(𝜪<sub>1,1</sub>), ..., Γ(𝜪<sub>k<sub>1</sub>,1</sub>)}, ..., {Γ(𝜪<sub>1,i</sub>), ..., Γ(𝜪<sub>k<sub>i</sub>,i</sub>)}, ..., {Γ(𝜪<sub>1,N</sub>), ..., Γ(𝜪<sub>k<sub>N</sub>,N</sub>)}}.
</pre>

where,

<pre>
Γ(𝜪<sub>r,i</sub>) = Τ(𝜪<sub>i</sub>) * Η<sub>r</sub>/Η<sub>TOTAL</sub>.
</pre>

### 4.1 Gain/Loss

```

```

## 5. Economic Implications

### 5.1 Market Efficiency

The protocol's equalisation mechanism promotes market efficiency through a novel approach to price discovery. The overlap calculation provides a sophisticated measure of position accuracy:

```

```

### 5.2 Incentive Structure

The protocol implements a reward structure that incentivises accurate position-taking. The return ratio for a position is given by:

```

```

## 6. Use Cases and Applications

### 6.1 Binary Outcome Markets

Consider a market predicting a binary outcome with the following parameters:

Example:

```typescript

```

The resulting `$BARYON` - `$PHOTON` pair would be:

```

```

### 6.2 Market Resolution

Upon market resolution with truth values `[0.6, 0.4]`, the equalisation function produces normalised returns:

```typescript

```

## 8. Conclusions and Future Work

The Collider Protocol presents a novel approach to binary outcome markets, introducing mathematical rigor through its `$BARYON` - `$PHOTON` mechanics and equalisation function. Future development could explore multi-outcome markets and dynamic truth value adjustment mechanisms.
