# 46-总之就是非常可爱

#### 介绍

TOPIC_ID:46, TEAM_ID:1498417324, TEAM_NAME:总之就是非常可爱.

layer_sword是一款镜像分割归档工具，用户可以将有效docker镜像归档文件(tar)作为输入，并分割成多个压缩子集(tar.gz)。

除此之外，layer_sword也支持将分割后的子集合并为等效归档文件(tar)，用于导入有效镜像。

##### 软件特色

* 任意分割：用户可以将归档文件分割为2、3、4......层，任意合理的分割方式软件都可以处理
* 配置文件：用户可以通过命令行参数或配置文件两种方式来指定分割方式
* 自动推导：所有分割层数中，允许最多一个`-1`项，软件会自动计算实际分割层数
* 自检查：分割后的子集，只需要放在同一个目录下，程序就可以自动完成检查及合并，无需额外的配置文件

#### 软件架构

layer_sword软件中，根目录下src目录为源码集合，tests目录为测试项目集合，Cargo.toml和Cargo.lock(自动生成)为Cargo项目构建描述文件。

##### 源码描述

| 文件名       | 描述                                     |
| ------------ | ---------------------------------------- |
| main.rs      | rust主程序入口，用于提供用户项相关功能   |
| lib.rs       | rust库入口，用于供单元测试项调用相关功能 |
| client.rs    | 命令行组件，用于解析命令和发起功能调用   |
| split.rs     | 完成分割操作的相关函数                   |
| merge.rs     | 完成合并操作的相关函数                   |
| inspector.rs | 完成镜像完整性检查的相关函数             |
| util.rs      | 杂项函数                                 |
| errors.rs    | 自定义错误类型集合                       |

##### 测试描述

| 文件名                       | 测试名                | 描述                     |
| ---------------------------- | --------------------- | ------------------------ |
| test_util.rs                 | test_string_sha256    | 测试字符串哈希函数       |
| [单元测试，测试工具函数输出] | test_file_sha256      | 测试文件哈希函数         |
|                              | test_stack_id         | 测试层叠哈希函数         |
| test_flow.rs                 | test_init_path        | 测试工作目录路径初始化   |
| [集成测试，测试工作流]       | test_inspect          | 测试镜像文件完整性检查   |
|                              | test_split_layer      | 测试分割功能             |
|                              | test_deduction        | 测试自动推导分割层数     |
|                              | test_split_four_layer | 测试分割为4层            |
|                              | test_split_two_layer  | 测试分割为2层            |
|                              | test_merge            | 测试合并功能             |
|                              | test_compress_best    | 测试压缩到`best`级别     |
| test_cmd.rs                  | test_split_basic      | 测试基本压缩命令         |
| [集成测试，测试命令行控制]   | test_split_negatives  | 测试带自动推导的压缩命令 |
|                              | test_split_config     | 测试用配置文件的压缩命令 |
|                              | test_merge_basic      | 测试基本合并命令         |
| test_err.rs                  | test_blank            | 测试空命令错误           |
| [集成测试，测试错误处理]     | test_conflicts        | 测试冲突命令错误         |



#### 安装教程

1.  从源码编译时，可以先clone项目到本地，移动到项目文件夹下，运行`Cargo run --bin layer_sword --release`构建项目，可执行文件是自动生成的release文件夹下的`layer_sword.exe`或`layer_sword`
2.  也可以使用随项目提供的可执行文件`layer_sword.exe`或`layer_sword`直接运行

#### 使用说明

1.  xxxx
2.  xxxx
3.  xxxx