from os import path

from setuptools import find_packages, setup

__version__ = "0.0.0"


# Read long description from README.md
here = path.abspath(path.dirname(__file__))
with open(path.join(here, "README.md"), encoding="utf-8") as readme:
    long_description = readme.read()


setup(
    name="axyn-matrix",
    version=__version__,
    description="A Matrix chatbot",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Daniel Thwaites",
    author_email="danthwaites30@btinternet.com",
    keywords="Matrix bot chatbot",
    url="https://github.com/danth/axyn-matrix",
    project_urls={
        "Bug Reports": "https://github.com/danth/axyn-matrix/issues",
        "Source": "https://github.com/danth/axyn-matrix",
    },
    classifiers=[
        "License :: OSI Approved :: GNU Affero General Public License v3",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
    ],
    packages=find_packages(),
    python_requires=">=3.6,<4",
    install_requires=[
        "flipgenic >=2.2,<3",
        "matrix-nio[e2e] >=0.18,<1",
    ],
    entry_points={
        "console_scripts": [
            "axyn_matrix=axyn_matrix.__main__:main",
        ],
    },
)
