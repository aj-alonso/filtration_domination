cmake_minimum_required(VERSION 3.9)
project(single_parameter)

set(CMAKE_CXX_STANDARD 20)

find_package(Boost REQUIRED)
include_directories(${Boost_INCLUDE_DIRS})

add_executable(single_parameter single_parameter_glisse_pritam.cpp)
target_link_libraries(single_parameter ${Boost_LIBRARIES})
