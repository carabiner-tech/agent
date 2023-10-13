def euler_problem_11():
    """Solves Project Euler Problem 11 by calculating and printing the greatest product of four adjacent numbers in the 20×20 grid."""
    # Read the grid from the 11.data file
    with open('test/data/11.data', 'r') as file:
        grid = [list(map(int, line.strip().split())) for line in file]
    
    max_product = 0
    rows = len(grid)
    cols = len(grid[0]) if rows > 0 else 0
    
    # Check horizontally
    for i in range(rows):
        for j in range(cols - 3):
            product = grid[i][j] * grid[i][j + 1] * grid[i][j + 2] * grid[i][j + 3]
            max_product = max(max_product, product)
    
    # Check vertically
    for i in range(rows - 3):
        for j in range(cols):
            product = grid[i][j] * grid[i + 1][j] * grid[i + 2][j] * grid[i + 3][j]
            max_product = max(max_product, product)
    
    # Check diagonally (left to right)
    for i in range(rows - 3):
        for j in range(cols - 3):
            product = grid[i][j] * grid[i + 1][j + 1] * grid[i + 2][j + 2] * grid[i + 3][j + 3]
            max_product = max(max_product, product)
    
    # Check diagonally (right to left)
    for i in range(3, rows):
        for j in range(cols - 3):
            product = grid[i][j] * grid[i - 1][j + 1] * grid[i - 2][j + 2] * grid[i - 3][j + 3]
            max_product = max(max_product, product)
    
    print(max_product)

if __name__ == "__main__":
    euler_problem_11()