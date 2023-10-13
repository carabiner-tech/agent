from utils import is_palindrome

def euler_problem_4():
    """Solves Project Euler Problem 4 by calculating and printing the largest palindrome made from the product of two 3-digit numbers."""
    max_palindrome = 0
    for i in range(100, 1000):
        for j in range(i, 1000):
            product = i * j
            if is_palindrome(product) and product > max_palindrome:
                max_palindrome = product
    print(max_palindrome)

if __name__ == "__main__":
    euler_problem_4()