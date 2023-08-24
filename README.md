[English](./READMEEN.md)

用rust实现的类似于webgpu标准的可编程软渲染管线


- [x] 基础数学库

- [x] 支持gltf模型

- [x] webgpu渲染管线标准实现

     - [X] 顶点解析
     - [X] 顶点着色器
     - [X] 图元裁剪（简单齐次裁剪）
     - [X] 图元组装（triangle-list）
     - [X] 光栅化
     - [X] 片元着色器
     - [X] 纹理支持
     - [X] 法线贴图
     - [ ] 多重采样


- [ ] Pbr

    - [X] 材料
    - [ ] 光源
      - [X] 点光源
      - [ ] 平行光
      - [ ] 聚光灯
    - [ ] 相机


- [X] bevy-like mesh 渲染
        
- [ ] 完整相机控制

- [ ] 阴影


## 截图

![三角形](./image_triangle.png)
![立方体](./image_mesh.png)
![robot](./image_pbr.png)



参考:

bevy 游戏引擎
https://github.com/bevyengine/bevy

webgpu 标准中关于渲染管线的算法
https://www.w3.org/TR/webgpu/#rendering-operations

闫令琪老师的games101（mua~ 大爱闫老师）
https://games-cn.org/intro-graphics/

韦易笑老师的软渲染器
https://github.com/skywind3000/RenderHelp

张林伟老师的用rust实现的软渲染器
https://github.com/NightsWatchGames/tiny-renderer

Filament 项目 中pbr实现原理
https://google.github.io/filament/Filament.html#figure_roughness

ue4官方的着色器模型实现
https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf