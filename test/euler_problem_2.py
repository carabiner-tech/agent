from utils import generate_fibonacci

def euler_problem_2(limit=4000000):
    """Solves Project Euler Problem 2 by calculating and printing the sum of all the even Fibonacci numbers below the specified limit."""
    total = sum(x for x in generate_fibonacci(limit) if x % 2 == 0)
    print(total)

if __name__ == "__main__":
    euler_problem_2()
