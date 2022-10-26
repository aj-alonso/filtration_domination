suppressPackageStartupMessages(library(tidyverse))
library(kableExtra, warn.conflicts = FALSE)

# Size of plot PDF.
aspect_ratio <- 10 / 30
width <- 10
height <- aspect_ratio * width

do_orders <- function() {
  ORDER_TIMEOUT <- 60 * 60 * 2

  orders_csv <- read.csv(file = "charts/compare_orders.csv") %>%
    mutate(Time = if_else(Time >= ORDER_TIMEOUT, NA_real_, Time)) %>%
    mutate(After = replace(After, is.na(Time), NA_integer_)) %>%
    mutate(Ratio = 1. - After / Before)

  order_table <- orders_csv %>%
    select(Dataset, Modality, Order, Ratio) %>%
    mutate(Modality = factor(Modality, c("Filtration-domination", "Strong filtration-domination", ordered = TRUE))) %>%
    mutate(Order = factor(Order, c("Rand", "Colex", "Lex", "RevColex", "RevLex"))) %>%
    mutate(Order = fct_recode(Order, Random = "Rand", Colexicographic = "Colex", Lexicographic = "Lex", "Reverse colex." = "RevColex", "Reverse lex." = "RevLex")) %>%
    mutate(Dataset = factor(Dataset, c("senate", "eleg", "netwsc", "hiv", "dragon", "sphere", "uniform", "circle", "torus", "swiss-roll"))) %>%
    arrange(Dataset, Modality, Order) %>%
    filter(Modality == "Filtration-domination") %>%
    group_by(Dataset) %>%
    mutate(Ratio = ifelse(Ratio == max(Ratio, na.rm = T), cell_spec(scales::percent(Ratio, accuracy = 0.2), "latex", bold = TRUE), scales::percent(Ratio, accuracy = 0.2, suffix = "\\%"))) %>%
    pivot_wider(names_from = Dataset, values_from = Ratio) %>%
    select(-Modality)
  options(knitr.kable.NA = '---')
  kbl(order_table, "latex",
      escape = F,
      booktabs = T,
      label = "order",
      caption = "Comparison of the edges removed when using different orders.
      For each dataset and order, we show the percentage of removed edges after a single run of the filtration-domination removal algorithm.
      The cases where the algorithm took more than 2 hours are marked with an ``---''.",
      align = c("l", rep("r", 10)),
      table.envir = "table*",
      position = "!h"
  ) %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 1, "Datasets" = ncol(order_table) - 1)) %>%
    cat(., file = "charts/compare_orders.tex")
}

do_removals <- function() {
  removals_csv <- read.csv(file = "charts/compare_removal.csv")

  # Create table.
  main <- removals_csv %>%
    pivot_wider(names_from = Policy, values_from = c(After, Time))
  main_selected <- main %>%
    select(Dataset, Before,
           "After_filtration-domination", "Time_filtration-domination",
           "After_strong-filtration-domination", "Time_strong-filtration-domination",
           "After_single-parameter", "Time_single-parameter")
  kbl(main_selected, "latex",
      booktabs = T,
      label = "removals",
      caption = "Performance evaluation. The first two columns describe the datasets.
      Each group of columns contains two subcolumns: ``After'', the number of resulting edges after running the corresponding algorithm, and ``Time (s)'', the time taken in seconds.",
      col.names = c("Datasets", "Before",
                    "After", "Time (s)",# "+par (s)",
                    "After", "Time (s)",# "+par (s)",
                    "After", "Time (s)"),
      table.envir = "table*") %>%
    kable_styling(latex_options = c("striped", "hold_position")) %>%
    add_header_above(c(" " = 2, "Filtration-domination" = 2, "Strong filtration-domination" = 2, "Single-parameter" = 2)) %>%
    cat(., file = "charts/compare_removal.tex")
}

do_random_densities <- function() {
  TIMEOUT <- 60 * 30
  all_random_densities <- read.csv(file = "charts/compare_random_densities.csv")

  random_densities <- all_random_densities %>%
    # Select reverse lexicographic orders.
    dplyr::filter(Order == "RevLex") %>%
    dplyr::filter(Structure == "no-densities" | Structure == "random") %>%
    mutate(Ratio = 1. - After / Before) %>%
    mutate(DominatedRatio = 1. - Dominated / Before) %>%
    mutate(Ratio = if_else(Time > TIMEOUT, NA_real_, Ratio)) %>%
    mutate(DominatedRatio = if_else(Time > TIMEOUT, NA_real_, DominatedRatio)) %>%
    mutate(Ratio = scales::percent(Ratio, accuracy = 0.2, suffix = "\\%")) %>%
    mutate(DominatedRatio = scales::percent(DominatedRatio, accuracy = 0.2, suffix = "\\%")) %>%
    select(Dataset, Structure, Ratio, DominatedRatio) %>%
    pivot_wider(names_from = Structure, values_from = c(Ratio, DominatedRatio)) %>%
    relocate(Dataset, "DominatedRatio_no-densities", "Ratio_no-densities", "DominatedRatio_random" , "Ratio_random")

  options(knitr.kable.NA = '---')
  kbl(random_densities, "latex",
      booktabs = T,
      escape = F,
      label = "random_densities",
      caption = "Analysis of the removed edges under
      changes to the structure of the grades. There are two groups of columns:
      one where we artificially zero out all the density values, and one where we
      replace them by random values sampled uniformly. ``Free at birth'' shows the
      percentage of edges that are not dominated when they appear (at their
      critical grade), and ``Removed'' is the percentage of edges removed after
      running our strong filtration-domination removal algorithm.",
      col.names = c("Dataset", "Free at birth", "Removed", "Free at birth", "Removed"),
      align = c("l", rep("r", 4)),
      table.envir = "table*",
      position = "!h"
  ) %>%
    kable_styling(latex_options = c("striped", "hold_position"), font_size = 9) %>%
    add_header_above(c(" " = 1, "Zeroed densities" = 2, "Random densities" = 2)) %>%
    cat(., file = "charts/compare_random_densities.tex")
}

format_kilobytes <- function(kb) {
  mb <- kb/1024.0
  gb <- kb/1024.0 ^ 2

  if (gb > 1) {
    formatted <- paste0(round(gb, 2), " GB")
  } else if (mb > 1) {
    formatted <- paste0(round(mb, 2), " MB")
  } else{
    formatted <- paste0(round(kb, 2), " KB")
  }

  return(formatted)
}

do_mpfree <- function() {
  mpfree_csv <- read.csv(file = "charts/compare_mpfree.csv", na.strings = c("NA", "-")) %>%
    mutate(Modality = factor(Modality, c("only-mpfree", "filtration-domination", "strong-filtration-domination"))) %>%
    mutate(Dataset = factor(Dataset, c("senate", "eleg", "netwsc", "hiv", "dragon",
                                 "sphere", "uniform", "circle", "torus", "swiss-roll"), ordered = TRUE)) %>%
    arrange(Dataset) %>%
    rowwise() %>%
    mutate(Total = sum(c(Collapse, Build, Mpfree), na.rm = TRUE))

  speedup_df <- mpfree_csv %>%
    mutate(Memory = mapply(format_kilobytes, Memory)) %>%
    group_by(Dataset) %>%
    arrange(Modality, .by_group = T) %>%
    mutate(Speedup = first(Total)/Total) %>%
    mutate(Speedup = replace(Speedup, Speedup == 0., NA)) %>%
    mutate(Speedup = if_else(is.na(Speedup), "---", format(round(Speedup, 2), nsmall = 2)))

  options(knitr.kable.NA = '---')
  hor_table <- speedup_df %>%
    select(Dataset, Points, Before, Modality, After, Collapse, Build, Mpfree, Speedup, Memory) %>%
    pivot_wider(names_from = Modality, values_from = c(Collapse, After, Build, Mpfree, Speedup, Memory)) %>%
    mutate(`Memory_only-mpfree` = if_else(`Speedup_strong-filtration-domination` == "---", "$\\infty$", `Memory_only-mpfree`)) %>%
    select(Dataset,
           "Memory_only-mpfree", "Build_only-mpfree", "Mpfree_only-mpfree",
           "Memory_strong-filtration-domination", "Collapse_strong-filtration-domination", "Build_strong-filtration-domination", "Mpfree_strong-filtration-domination", "Speedup_strong-filtration-domination")
  kbl(hor_table, "latex",
      digits = 2,
      escape = FALSE,
      booktabs = T,
      label = "mpfree",
      caption = "Impact of our algorithm as a preprocessing step for minimal presentations.
      Inside each group of columns, the ``Build (s)'' column displays the time taken in seconds to build the clique bifiltration, ``mpfree (s)'' the time taken to run \\texttt{mpfree},
      and ``Memory'' the maximum amount of memory used by the pipeline, over all the steps (including the preprocessing if applied).
      In addition, the ``Removal (s)'' column displays the time taken to run our algorithm, and ``Speedup'' is the speedup compared to not doing preprocessing. The $\\infty$ symbol means
      that the pipeline ran out of memory, and in that
      case both the timing and speedup values are marked with an ``---''.",
      col.names = c("Dataset",
                    "Memory", "Build (s)", "mpfree (s)",
                    "Memory", "Removal (s)", "Build (s)", "mpfree (s)", "Speedup"),
      align = c("l", rep("r", 6)),
  table.envir = "table*", position = "!h") %>%
    kable_styling(latex_options = c("striped", "hold_position")) %>%
    add_header_above(c(" " = 1,
                       "No preprocessing" = 3,
                       "With preprocessing" = 4)) %>%
    cat(., file = "charts/compare_mpfree.tex")
}

do_multiple_iterations <- function() {
  multiple_iters_csv <- read.csv(file = "charts/compare_multiple_iterations.csv") %>%
    group_by(Dataset) %>%
    mutate(Ratio = Edges/first(Edges)) %>%
    dplyr::filter(Iteration <= 5)

  iters <- 1:5
  get_col <- \(n) c(sprintf("Time_%d", n), sprintf("Ratio_%d", n))

  iters_table <- multiple_iters_csv %>%
    dplyr::filter(Iteration > 0) %>%
    select(-Edges) %>%
    mutate(Ratio = 1 - Ratio) %>%
    mutate(Ratio = coalesce(Ratio - lag(Ratio, n = 1), Ratio)) %>%
    mutate(Time = coalesce(Time - lag(Time, n = 1), Time)) %>%
    mutate(Ratio = scales::percent(Ratio, accuracy = 0.2, suffix = "\\%")) %>%
    pivot_wider(names_from = Iteration, values_from = c(Ratio, Time)) %>%
    relocate(c(Dataset, unlist(iters %>% purrr::map(get_col))))

  kbl(iters_table, "latex",
      digits = 2,
      escape = FALSE,
      booktabs = T,
      label = "iterations",
      caption = "Results after running the strong filtration-domination removal algorithm 5 consecutive times.
      There are 5 groups of columns, one for each iteration.
      The ``Removed'' column displays the percentage of the original edges removed in the corresponding iteration,
      and ``Time (s)'' displays the running time (in seconds) of the iteration.",
      align = c("l", rep("r", 10)),
      col.names = c("Dataset", rep(c("Time (s)", "Removed"), 5)),
  table.envir = "table*", position = "!h") %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 1,
                       "Iteration 1" = 2,
                       "Iteration 2" = 2,
                       "Iteration 3" = 2,
                       "Iteration 4" = 2,
                       "Iteration 5" = 2
                       )) %>%
    cat(., file = "charts/compare_multiple_iterations.tex")
}

do_asymptotics <- function() {
  asymptotics_csv <- read.csv(file = "charts/compare_asymptotics.csv") %>%
    mutate(Ratio = (After / Before) * 100) %>%
    mutate(Vertices = Points * Points)

  asympt_width <- 4

  torus_csv <- asymptotics_csv %>%
    dplyr::filter(Dataset == "torus" & Algorithm == "Strong filtration-domination")
  ggplot(torus_csv,
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Torus") +
    geom_smooth(method = "lm", formula = y ~ poly(x, 2), se = FALSE) +
    geom_point()

  ggsave("charts/compare_asymptotics_torus.pdf", width = asympt_width, height = asympt_width)

  uniform_csv <- asymptotics_csv %>%
    dplyr::filter(Dataset == "uniform" & Algorithm == "Strong filtration-domination")
  ggplot(uniform_csv,
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Uniform") +
    geom_smooth(method = "lm",
                formula = y ~ poly(x, 2),
                se = FALSE) +
    geom_point()

  ggsave("charts/compare_asymptotics_uniform.pdf", width = asympt_width, height = asympt_width)
}

commands <- c("orders", "removal", "mpfree", "multiple-iterations", "asymptotics", "random-densities")

# Use all commands by default, unless some are given as arguments.
args <- commandArgs(trailingOnly=TRUE)
if (length(args) > 0) {
  commands <- args
}

for (command in commands) {
  if (command == "orders") {
    do_orders()
  } else if (command == "removal") {
    do_removals()
  } else if (command == "mpfree") {
    do_mpfree()
  } else if (command == "multiple-iterations") {
    do_multiple_iterations()
  } else if (command == "asymptotics") {
    do_asymptotics()
  } else if (command == "random-densities") {
    do_random_densities()
  } else {
    stop("Unknown command.", call.= FALSE)
  }
}
