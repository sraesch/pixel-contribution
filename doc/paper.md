# Estimate screen coverage of 3D objects

## Abstract
For progressive loading approaches, it is essential to prioritize the loading of objects that are visible on the screen. The question of priority is closely linked to the screen coverage of an object.
The more an object is visible on the screen, the more it should be loaded first.
In this paper, we propose a method to estimate the screen coverage of 3D objects in a scene. The method is based on a two stage approach. First, we estimate the coverage of the bounding volume of the object. Then, in the second step, we refine the estimation by using precomputed values of the screen coverage of the actual object.

## 1. Introduction
In this paper, we investigate the problem of estimating the screen coverage of 3D objects in a scene based on precomputed values. The model $M$ consisting of the precomputed values should be moderate in size. Moreover, it should be fast to query the model $M$ for a given object. The model $M$ should be able to estimate the screen coverage of an object in a scene with a good approximation.

```mermaid
flowchart LR
    A[Modelview m, Projection p, width, height] -->|Query| B(Screen Coverage Model: M)
    B -->|Return| C[Screen Coverage in Pixels]
```
Figure 1: Querying the model $M$ for the screen coverage of an object using the modelview matrix $m$, the projection matrix $p$, the width and height of the screen.


As illustrated in Figure 1, the model $M$ is queried with the modelview matrix $m$, the projection matrix $p$, and the width and height of the screen. The model $M$ returns the screen coverage of the object in pixels. The modelview matrix $m$ is the transformation matrix that transforms the object from object space to world space. The projection matrix $p$ is the transformation matrix that transforms the object from world space to clip space. The width and height of the screen are the dimensions of the screen in pixels. The model $M$ is precomputed for each object in the scene.

The model is defined as $M:=(B, P)$, where $B$ is the bounding volume of the object and $P$ is the precomputed screen coverage of the object. The bounding volume $B$ is a bounding sphere and is defined as $B:=(c, r)$, where $c$ is the center of the bounding sphere and $r > 0$ is the radius of the bounding sphere. The precomputed screen coverage $P$ is a little bit more complex and will be explained in [3. Deriving the screen coverage model $P$ of the object](#3-deriving-the-screen-coverage-model-of-the-object).

## 2. Estimating the screen coverage of the bounding sphere $B$
We first focus on the estimation of the screen coverage of the bounding sphere $B$. The bounding sphere $B$ is defined as $B:=(c, r)$, where $c$ is the center of the bounding sphere and $r > 0$ is the radius of the bounding sphere. The screen coverage of the bounding sphere $B$ is the number of pixels that the bounding sphere $B$ covers on the screen.

First, we transform the center $c$ of the bounding sphere $B$ into the view space of the camera by transforming it with the modelview matrix $m$.
$$
c' = m \cdot (c, 1)
$$
$c'$ is the transformed center of the bounding sphere $B$ in view space. The $1$ has been added to the center $c$ to make it a point in homogeneous coordinates.

First we handle the special case where $c^T \cdot c \leq r^2$. In this case, the camera is inside the bounding sphere and we return $width \cdot height$ as the screen coverage of the bounding sphere $B$.

## 3. Deriving the screen coverage model $P$ of the object

## 4. Combined two stage model $M$

## 5. Testing the model $M$

## 6. Conclusion