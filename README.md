# 项目功能说明书

[TOC]

## 介绍

### 背景介绍

在使用docker等工具时，用户一般从镜像仓库运行`docker pull`，从镜像仓库拉取镜像到本地，再创建容器使用。但是，在某些特殊场景下，如镜像过大，网络连接不佳，或者处于无网络环境时，我们不会使用镜像仓库，而是通过`docker save`+`docker load`（或者`isula-build save`+`isula-build load`）的方式导入镜像。这两条命令中，前者会将本机上已存在的镜像打包为归档文件`tar`，后者会以有效的归档文件`tar`作为输入，将镜像导入本机。

需要注意到，很多镜像都存在相同的基础组件，因此也会共享数个相同的层。这是因为在`image`制作时，它们都由相同的`image`加工而来。

在功能上，我们一般可以将`image`中的层进行简单的分类：提供操作系统的`os`（如`ubuntu`、`openeuler`），所依赖的运行组件`lib`（如`Python`、`JDK`），以及实现具体功能的`app`，一般情况下，这三类将会由底层到上层依次排布，且较上的类必须依赖下层的类。

因此，在这种场景下，我们可以使用一种办法来减少镜像体积并分类归档：我们可以将每个`image`都分为数个分割子集，`os`、`lib`、`app`或更多，然后将其分别压缩并归档。这样的话，重复的分割子集将会只保存一次，能够大大节省储存空间。

### 产出描述

基于以上目标，我们构建了`layer_sword`，用于解决这个问题。

这是一款镜像分割归档工具，用户可以将有效`docker`等镜像归档文件`tar`作为输入，并分割成多个压缩子集`tar.gz`。

除此之外，`layer_sword`也支持将分割后的子集合并为等效归档文件`tar`，用于导入有效镜像。

### 软件特色

* 任意分割：用户可以将归档文件分割为2、3、4......层，任意合理的分割方式软件都可以处理
* 配置文件：用户可以通过命令行参数或配置文件两种方式来指定分割方式
* 自动推导：所有分割层数中，允许最多一个`-1`项，软件会自动计算实际分割层数
* 自检查：分割后的子集，只需要放在同一个目录下，程序就可以自动完成检查及合并，无需额外的配置文件



## 安装教程

### 安装说明

1.  从源码编译时，可以先clone项目到本地，移动到项目文件夹下，运行`Cargo run --bin layer_sword --release`构建项目，可执行文件是自动生成的release文件夹下的`layer_sword.exe`（Windows）或`layer_sword`（Linux）
2.  也可以使用随项目提供的可执行文件`layer_sword.exe`（Windows）或`layer_sword`（Linux）直接运行

### 使用流程

1.  准备好本机上的镜像，或者使用`docker pull [mirror name]`或类似命令拉取镜像
2.  使用`docker save -o [mirror.tar] [mirror name]`或类似命令将指定镜像保存成`tar`归档文件
3.  利用`layer_sword split`命令对`tar`归档文件进行分割，并对获得的压缩子集分别进行对应归档
4.  利用`layer_sword merge`命令对`tar.gz`分割子集进行合并，获得等效原始tar归档文件
5.  使用`docker load -i [mirror.tar]`或类似命令将`tar`归档文件进行导入

## 使用教程

### 命令介绍

**split子命令**

| 参数     | 简称 | 取值                  | 描述                                 | 强制                     |
| -------- | ---- | --------------------- | ------------------------------------ | ------------------------ |
| --config | -c   | \<FILE\>              | 从用户指定的配置文件获得分割信息     | 和[name && layers]二选一 |
| --names  | -n   | \<STR, STR...\>       | 指定分割各子集名称                   | 和[config]二选一         |
| --layers | -l   | \<INT, INT...\>       | 指定分割各子集含有层数量             | 和[config]二选一         |
| --target | -t   | \<FILE\>              | 指定镜像归档文件路径                 | 是                       |
| --output | -o   | \<DIRECTORY\>         | 指定的子集输出路径                   | 否，默认值`./out`        |
| --work   | -w   | \<DIRECTORY\>         | 指定的工作临时文件夹                 | 否，默认值`./tmp`        |
| --level  | -v   | 0-9, none, fast, best | 指定分割子集压缩等级，越大压缩率越高 | 否，默认值6              |
| --quiet  | -q   | 无                    | 启用时，程序静默运行，不输出信息     |                          |

**merge子命令**

| 参数     | 简称 | 取值          | 描述                             | 强制              |
| -------- | ---- | ------------- | -------------------------------- | ----------------- |
| --target | -t   | \<DIRECTORY\> | 指定分割子集所在文件夹路径       | 是                |
| --output | -o   | \<DIRECTORY\> | 指定的子集输出路径               | 否，默认值`./out` |
| --work   | -w   | \<DIRECTORY\> | 指定的工作临时文件夹             | 否，默认值`./tmp` |
| --quiet  | -q   | 无            | 启用时，程序静默运行，不输出信息 |                   |

### 配置文件

配置文件为`json`格式，需求`names`和`layers`两个数组条目，与`split`子命令中的同名参数等效。

典型的配置文件内容如下：

```
{
    "names": [
        "os", 
        "lib", 
        "app"
    ], 
    "layers": [
        1, 
        -1, 
        1
    ]
}
```

### 情景示例

`layer_sword split -n os,lib,app -l 1,3,1 -t base.tar`

将`base.tar`镜像归档文件自底向上分为`os`、`lib`和`app`三个压缩子集，分别含有1层、3层、1层layer。临时工作目录为当前目录下的`tmp`文件夹（默认），输出文件在当前目录下的`out`文件夹（默认）。

`layer_sword split -n os,lib -l 1,-1 -t base.tar -s -w work`

将`base.tar`镜像归档文件自底向上分为`os`和`lib`两个压缩子集，前者含有1层layer，后者含有剩余所有层layer。除此之外，运行过程中不输出提示信息。临时工作目录为当前目录下的`work`文件夹（用户指定），输出文件在当前目录下的out文件夹（默认）。

`layer_sword split -c config.json -t base.tar -o splits`

将`base.tar`镜像归档文件根据config.json配置文件中的信息分割为压缩子集。临时工作目录为当前目录下的`tmp`文件夹（默认），输出文件在当前目录下的`splits`文件夹（用户指定）。

`layer_sword merge -t splits`

将`splits`文件夹下所有的分割子集合并为等效镜像归档文件。临时工作目录为当前目录下的`tmp`文件夹（默认），输出文件在当前目录下的splits文件夹（用户指定）。



## 技术细节

### 排序方案

layer_sword进行分割时，将用以下方案进行自排序：

每个分割子集中的`split_config.json`文件会记录当前序列号index，越低为越底层的子集，越高为越上层的子集，其中：

$index\in[0, len(splits)-1]$

### 验证方案

layer_sword进行分割时，将用以下方案进行自验证：

1. 每个分割子集`tar.gz`文件的备注中，会记录内部`tar`文件的`sha256`

2. 每个分割子集中的`split_config.json`文件会记录父级id和层叠id

   父级id计算方式如下

   $parrent\_id=sha256(parrent\_layer\_file)$

   层叠id计算方式如下

   $stack\_id(0)= sha256(""+"\n"+"")$

   $stack\_id(i) = sha256(stack\_id(i-1)+"\n"+sha256(parrent\_layer\_file))$

分割子集合并时，会验证以上所有id，以确认子集不存在错误

### 一致性方案

1. 在`tar`压缩方案中，压缩文件内部文件元数据（如时间）将会影响压缩文件哈希，为了消除这种影响，执行压缩时将会忽略所有文件元数据。
2. 在`tar`压缩方案中，压缩文件内部文件顺序将会影响压缩文件哈希。相应的，由于不同平台中默认文件读取顺序不一致，程序将在读取所有文件后，进行排序再压缩。

### 拓展方案

关于项目中分割验证方案和检查方案，我们提供了抽象接口用于未来可能的拓展，具体方案如下：

分割合并中的验证方案由`dominator`文件夹中的控制器文件`BaseDominator`提供，如果需要构建新的`Xdominator`，用户可以在其中创建新的控制器文件，在其中：

1. 定义配置类`Xconfig`，对其`impl Config trait`并重写所有方法
2. 定义控制类`XDominator`，对其`impl Split trait`并重写`pack_tar_with_config`方法用于配置文件的生成和打包`tar`，再`impl Merge trait`并重写`check_with_config`方法用于配置文件的验证和`init_config`方法用于配置类对象`Xconfig`的初始化

镜像文件的检查方案由`inspector`文件夹中的检查器`BaseInspector`提供，如果需要构建新的`XInspector`，用户可以在其中创建新的检查器文件，在其中：

1. 定义检查类`XInspector`，对其`impl Inspect trait`并重写所有局部检查方法`inspect_route`，`inspect_config`，`inspect_layer`，`inspect_manifest`，用于实现各个检查过程

完成新的拓展类构建后，在`client.rs`中的`pick_dominator_and_inspector`函数里，将新构建的拓展类用`Box`指针作为返回值，并调整返回不同控制器和检查器的逻辑。

## 备注

### 功能完成度

| 功能            | 项目要求 | 完成程度 |
| --------------- | -------- | -------- |
| 支持docker      | √        | √        |
| 支持isulad      | √        | √        |
| 支持isula-build | √        | √        |
| 完整性校验      | √        | √        |
| 有效输入校验    | √        | √        |
| 原生rust        | √        | √        |
| 配置文件启动    |          | √        |
| 自校验和自排序  |          | √        |
| 自动推导        |          | √        |
| 任意分割        |          | √        |

### 软件架构

layer_sword软件中，根目录下src目录为源码集合，tests目录为测试项目集合，Cargo.toml和Cargo.lock(自动生成)为Cargo项目构建描述文件。

#### 源码描述

| 文件名       | 描述                                     |
| ------------ | ---------------------------------------- |
| main.rs      | rust主程序入口，用于提供用户项相关功能   |
| lib.rs       | rust库入口，用于供单元测试项调用相关功能 |
| client.rs    | 命令行组件，用于解析命令和发起功能调用   |
| split.rs     | 完成分割操作的相关函数                   |
| merge.rs     | 完成合并操作的相关函数                   |
| inspector.rs | 完成镜像完整性检查的相关函数             |
| util.rs      | 工具类函数                               |
| errors.rs    | 自定义错误类型集合                       |

#### 测试描述

| 文件名                       | 测试名                   | 描述                     |
| ---------------------------- | ------------------------ | ------------------------ |
| test_util.rs                 | test_string_sha256       | 测试字符串哈希函数       |
| [单元测试，测试工具函数输出] | test_file_sha256         | 测试文件哈希函数         |
|                              | test_stack_id            | 测试层叠哈希函数         |
| test_flow.rs                 | test_init_path           | 测试工作目录路径初始化   |
| [集成测试，测试工作流]       | test_inspect             | 测试镜像文件完整性检查   |
|                              | test_split_layer         | 测试分割功能             |
|                              | test_deduction           | 测试自动推导分割层数     |
|                              | test_split_four_layer    | 测试分割为4层            |
|                              | test_split_two_layer     | 测试分割为2层            |
|                              | test_merge               | 测试合并功能             |
|                              | test_compress_best       | 测试压缩到`best`级别     |
| test_cmd.rs                  | test_split_basic         | 测试基本压缩命令         |
| [集成测试，测试命令行控制]   | test_split_negatives     | 测试带自动推导的压缩命令 |
|                              | test_split_config        | 测试用配置文件的压缩命令 |
|                              | test_merge_basic         | 测试基本合并命令         |
| test_err.rs                  | test_blank               | 测试空命令错误           |
| [集成测试，测试错误处理]     | test_split_conflict      | 测试冲突命令错误         |
|                              | test_split_no_info       | 测试无分割信息错误       |
|                              | test_split_no_target     | 测试无分割目标错误       |
|                              | test_merge_no_target     | 测试无合并目标错误       |
|                              | test_split_bad_extension | 测试分割目标错误后缀     |
|                              | test_split_bad_info      | 测试分割信息错误         |

### 覆盖率测试

**82%**（利用tarpaulin检测）

```
INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| src/client.rs: 21-22, 25, 41-45, 47, 57-58, 62-63, 67-68, 76, 79, 81, 93-94, 98, 101, 107, 111, 115-118, 133-134, 141-142, 149-150, 165-167, 182-184, 188-190, 324-326, 343-344, 352, 369-372, 398-399
|| src/dominator/base.rs: 34-35, 37-38, 44, 51-52, 82-84, 87-89
|| src/errors.rs: 77-80, 82-85, 107, 122-124, 144, 159-160, 162-163, 187-188, 190-191
|| src/inspector/base.rs: 30, 34-36, 53-55, 68, 70, 72, 87-88, 91, 96-98, 106-108, 116-117, 123-125, 138-140, 156-157, 159-161, 177-180, 185-186, 193-194, 197-198, 204, 207-208, 219-220, 227-229, 234-236, 243-245, 252-254, 259-261, 268-270, 276, 278, 282-285
|| src/inspector.rs: 44
|| src/main.rs: 15-19
|| src/merge.rs: 23-25, 50, 56-58, 77, 94, 123-125, 136, 141, 162, 165
|| src/split.rs: 30-32, 37-39, 50, 56-59, 89, 111, 146-147, 183, 197, 201, 205, 208-209
|| src/util.rs: 50, 62, 171-172, 204-205, 212-215, 251-252, 254
|| src/validator.rs: 25, 52
|| tests/common.rs: 9
|| tests/test_err.rs: 22, 48, 73, 98, 122, 148, 175
|| Tested/Total Lines:
|| src/client.rs: 153/208
|| src/dominator/base.rs: 58/71
|| src/errors.rs: 10/31
|| src/inspector/base.rs: 115/186
|| src/inspector.rs: 11/12
|| src/main.rs: 0/5
|| src/merge.rs: 102/118
|| src/split.rs: 111/132
|| src/util.rs: 117/130
|| src/validator.rs: 12/14
|| tests/common.rs: 9/10
|| tests/test_cmd.rs: 64/64
|| tests/test_err.rs: 82/89
|| tests/test_flow.rs: 207/207
|| tests/test_util.rs: 7/7
||
82.40% coverage, 1058/1284 lines covered
```

### 压力测试

我们选取了来自英伟达的镜像`nvcr.io/nvidia/tensorrt:21.02-py3`，这个镜像是人工智能模型推理加速工具`tensorRT`的官方镜像，有`5.7GB`的大小和`43`的层数，非常适合测试`layer_sword`对于超大镜像的处理性能。测试运行时，采用了最常见的`os`、`lib`、`app`分别为1，-1，1层的分割方案。

**split测试**

| 压缩等级 | 别名    | 耗时      | 处理速度    |
| -------- | ------- | --------- | ----------- |
| 0        | none    | `3m4.5s`  | `31.64MB/s` |
| 1        | fast    | `2m31.8s` | `38.45MB/s` |
| 6        | default | `6m32.2s` | `14.88MB/s` |
| 9        | best    | `9m27.9s` | `10.27MB/s` |

**merge测试**

其中`大小`为`split`分割后所有分割子集的文件大小之和

| 来源    | 大小   | 压缩率   | 耗时      | 处理速度    |
| ------- | ------ | -------- | --------- | ----------- |
| none    | `5.8G` | `101.8%` | `1m47.8s` | `55.09MB/s` |
| fast    | `3.3G` | `57.9%`  | `2m28.6s` | `22.74MB/s` |
| default | `3.0G` | `52.6%`  | `2m23.9`  | `21.35MB/s` |
| best    | `3.0G` | `52.6%`  | `2m24.8`  | `21.22MB/s` |
