def euler_problem_6(limit=100):
    """Solves Project Euler Problem 6 by calculating and printing the difference between the sum of the squares of the first one hundred natural numbers and the square of the sum."""
    sum_of_squares = sum(i**2 for i in range(1, limit + 1))
    square_of_sum = sum(range(1, limit + 1)) ** 2
    print(square_of_sum - sum_of_squares)

if __name__ == "__main__":
    euler_problem_6()