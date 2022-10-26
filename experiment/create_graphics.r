library(ggplot2)
library(tidyr)
library(dplyr, warn.conflicts = FALSE)
library(xtable)
library(kableExtra)
library(forcats)
library(formattable)
library(rlist)

# Size of plot PDF.
aspect_ratio <- 10 / 30
width <- 10
height <- aspect_ratio * width

do_orders <- function() {
  ORDER_TIMEOUT <- 60 * 60 * 2

  orders_csv <- read.csv(file = "compare_orders.csv") %>%
    mutate(Time = if_else(Time >= ORDER_TIMEOUT, NA_real_, Time)) %>%
    mutate(After = replace(After, is.na(Time), NA_integer_)) %>%
    mutate(Ratio = 1. - After / Before)

  order_table <- orders_csv %>%
    select(Dataset, Modality, Order, Ratio) %>%
    mutate(Modality = fct_relevel(Modality, "Filtration-domination", "Strong filtration-domination")) %>%
    mutate(Order = fct_relevel(Order, "Rand", "Colex", "Lex", "RevColex", "RevLex")) %>%
    mutate(Order = fct_recode(Order, Random = "Rand", Colexicographic = "Colex", Lexicographic = "Lex", "Reverse colex." = "RevColex", "Reverse lex." = "RevLex")) %>%
    mutate(Dataset = fct_relevel(Dataset, "senate", "eleg", "netwsc", "hiv", "dragon", "sphere", "uniform", "circle", "torus", "swiss roll")) %>%
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
      The cases where the algorithm took more than 2 hours are marked with an ---.",
      align = c("l", rep("r", 10))
  ) %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 1, "Datasets" = 10)) %>%
    cat(., file = "compare_orders.tex")
}

do_removals <- function() {
  removals_csv <- read.csv(file = "compare_removal.csv")

  # Create table.
  main <- removals_csv %>%
    pivot_wider(names_from = Policy, values_from = c(After, Time))
  main_selected <- main %>%
    select(Dataset, Before,
           "After_Geom", "Time_Geom",# "Time_Geom Par",
           "After_Single Vertex", "Time_Single Vertex",# "Time_Single Vertex Par",
           "After_Glisse", "Time_Glisse")
  kbl(main_selected, "latex",
      booktabs = T,
      label = "removals",
      caption = "Performance evaluation. The first two columns describe the datasets.
      Each group of columns contains two subcolumns: ``After'', the number of resulting edges after running the corresponding algorithm, and ``Time (s)'', the time taken in seconds.",
      col.names = c("Datasets", "Before",
                    "After", "Time (s)",# "+par (s)",
                    "After", "Time (s)",# "+par (s)",
                    "After", "Time (s)")) %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 2, "Filtration-domination" = 2, "Strong filtration-domination" = 2, "Single-parameter" = 2)) %>%
    cat(., file = "compare_removal.tex")
  df <- removals_csv %>%
    select(-Points) %>%
    mutate(Time = paste0("(", Time, "s)")) %>%
    unite(After, After, Time, sep = ' ') %>%
    pivot_wider(names_from = Policy, values_from = After)
  kbl(df, "latex", booktabs = T,
      label = "removals_full",
      caption = "Removal comparisons, full table.") %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    cat(., file = "compare_removal_full.tex")
}

do_removals_presentation <- function() {
  removals_csv <- read.csv(file = "compare_removal.csv")

  # Create table.
  main <- removals_csv %>%
    mutate(Ratio = 1. - After / Before) %>%
    select(Dataset, Ratio, Time, Policy) %>%
    mutate(Ratio = scales::percent(Ratio, accuracy = 0.2, suffix = "%")) %>%
    pivot_wider(names_from = Policy, values_from = c(Ratio, Time))

  plot_data <- removals_csv %>%
    mutate(Ratio = 1. - After / Before) %>%
    select(Dataset, Ratio, Time, Policy)
  ggplot(plot_data, aes(y = Ratio, x = Dataset, fill = Policy)) +
    geom_bar(position = "dodge", stat = "identity", width = 0.5) +
    labs(x = "Dataset", y = "Removed edges") +
    scale_y_continuous(labels = scales::percent) +
    theme(legend.position="bottom")
  ggsave("chart_presentation_removals.pdf", width = 10, height = 10 * 1/3)

  main_selected <- main %>%
    select(Dataset,
           "Ratio_Geom", "Time_Geom",# "Time_Geom Par",
           "Ratio_Single Vertex", "Time_Single Vertex",# "Time_Single Vertex Par",
           "Ratio_Glisse", "Time_Glisse")
  kbl(main_selected, "latex",
      booktabs = T,
      label = "removals",
      col.names = c("Datasets",
                    "Ratio", "Time (s)",# "+par (s)",
                    "Ratio", "Time (s)",# "+par (s)",
                    "Ratio", "Time (s)")) %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 1, "Filt.-dom." = 2, "Strong filt.-dom." = 2, "Single-parameter" = 2)) %>%
    cat(., file = "chart_presentation_removals.tex")
}

do_orders_presentation <- function() {
  ORDER_TIMEOUT <- 60 * 60 * 2

  orders_csv <- read.csv(file = "compare_orders.csv") %>%
    mutate(Time = if_else(Time >= ORDER_TIMEOUT, NA_real_, Time)) %>%
    mutate(After = replace(After, is.na(Time), NA_integer_)) %>%
    mutate(Ratio = 1. - After / Before)

  plot_data <- orders_csv %>%
    select(Dataset, Modality, Order, Ratio) %>%
    mutate(Modality = fct_relevel(Modality, "Filtration-domination", "Strong filtration-domination")) %>%
    mutate(Order = fct_relevel(Order, "Rand", "Colex", "Lex", "RevColex", "RevLex")) %>%
    mutate(Order = fct_recode(Order, Random = "Rand", Colexicographic = "Colex", Lexicographic = "Lex", "Reverse colex." = "RevColex", "Reverse lex." = "RevLex")) %>%
    mutate(Dataset = fct_relevel(Dataset, "senate", "eleg", "netwsc", "hiv", "dragon", "sphere", "uniform", "circle", "torus", "swiss roll")) %>%
    arrange(Dataset, Modality, Order) %>%
    filter(Modality == "Filtration-domination")
  ggplot(plot_data, aes(y = Ratio, x = Dataset, fill = Order)) +
    geom_bar(position = "dodge", stat = "identity", width = 0.5) +
    labs(x = "Dataset", y = "Removed edges") +
    scale_y_continuous(labels = scales::percent) +
    theme(legend.position="bottom")
  ggsave("chart_presentation_orders.pdf", width = 10, height = 10 * 1/3)
}

do_mpfree_presentation <- function() {
  mpfree_csv <- read.csv(file = "compare_mpfree.csv", na.strings = c("NA", "-")) %>%
    mutate(Modality = fct_relevel(Modality, "Only mpfree", "Collapse", "Strong collapse")) %>%
    mutate(Dataset = fct_relevel(Dataset, "senate", "eleg", "netwsc", "hiv", "dragon",
                                 "uniform", "torus", "swiss roll", "sphere", "circle")) %>%
    arrange(Dataset) %>%
    rowwise() %>%
    mutate(Total = sum(c(Collapse, Build, Mpfree), na.rm = TRUE))

  speedup_df <- mpfree_csv %>%
    group_by(Dataset) %>%
    arrange(Modality, .by_group = T) %>%
    mutate(Speedup = first(Total)/Total) %>%
    mutate(Speedup = replace(Speedup, Speedup == 1, NA)) %>%
    mutate(Speedup = replace(Speedup, Speedup == 0., NA)) %>%
    mutate(Speedup = if_else(is.na(Speedup), "$\\infty$", format(round(Speedup, 2), nsmall = 2))) %>%
    filter(Modality == "Strong collapse")

  options(knitr.kable.NA = '$\\infty$')
  hor_table <- speedup_df %>%
    select(Dataset, Speedup) %>%
    select(Dataset,
           Speedup)
  kbl(hor_table, "latex",
      digits = 2,
      escape = FALSE,
      booktabs = T,
      label = "mpfree",
      col.names = c("Dataset",
                    "Speedup"),
      align = c("l", "r")) %>%
    kable_styling(latex_options = c("striped", "hold_position")) %>%
    cat(., file = "chart_presentation_mpfree.tex")
}

do_random_densities <- function() {
  TIMEOUT <- 60 * 30
  all_random_densities <- read.csv(file = "compare_random_densities.csv")

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
      align = c("l", rep("r", 4))) %>%
    kable_styling(latex_options = c("striped", "hold_position"), font_size = 9) %>%
    add_header_above(c(" " = 1, "Zeroed densities" = 2, "Random densities" = 2)) %>%
    cat(., file = "compare_random_densities.tex")
}

do_preliminary <- function() {
  preliminary_csv <- read.csv(file = "compare_preliminary.csv") %>%
    mutate(Ratio = After / Before) %>%
    rename(Algorithms = Modality) %>%
    mutate(Algorithms = replace(Algorithms, Algorithms == "Ours", "Our approach on bifiltrations")) %>%
    mutate(Algorithms = replace(Algorithms, Algorithms == "Giotto", "Single-parameter"))

  ggplot(preliminary_csv, aes(y = Ratio, x = Dataset, fill = Algorithms)) +
    geom_bar(position = "dodge", stat = "identity", width = 0.5) +
    labs(x = "Datset", y = "Remaining edges") +
    scale_y_continuous(labels = scales::percent)

  ggsave("compare_preliminary.pdf", width = 8, height = 5 * aspect_ratio)
}

do_mpfree <- function() {
  mpfree_csv <- read.csv(file = "compare_mpfree.csv", na.strings = c("NA", "-")) %>%
    mutate(Modality = fct_relevel(Modality, "Only mpfree", "Collapse", "Strong collapse")) %>%
    mutate(Dataset = fct_relevel(Dataset, "senate", "eleg", "netwsc", "hiv", "dragon",
                                 "sphere", "uniform", "circle", "torus", "swiss roll")) %>%
    arrange(Dataset) %>%
    rowwise() %>%
    mutate(Total = sum(c(Collapse, Build, Mpfree), na.rm = TRUE))

  speedup_df <- mpfree_csv %>%
    group_by(Dataset) %>%
    arrange(Modality, .by_group = T) %>%
    mutate(Speedup = first(Total)/Total) %>%
    mutate(Speedup = replace(Speedup, Speedup == 1, NA)) %>%
    mutate(Speedup = replace(Speedup, Speedup == 0., NA)) %>%
    mutate(Speedup = if_else(is.na(Speedup), "---", format(round(Speedup, 2), nsmall = 2)))

  options(knitr.kable.NA = '$\\infty$')
  hor_table <- speedup_df %>%
    select(Dataset, Points, Before, Modality, After, Collapse, Build, Mpfree, Speedup) %>%
    pivot_wider(names_from = Modality, values_from = c(Collapse, After, Build, Mpfree, Speedup)) %>%
    select(Dataset,
           "Build_Only mpfree", "Mpfree_Only mpfree",
           # "Collapse_Collapse", "Build_Collapse", "Mpfree_Collapse", "Speedup_Collapse",
           "Collapse_Strong collapse", "Build_Strong collapse", "Mpfree_Strong collapse", "Speedup_Strong collapse")
  kbl(hor_table, "latex",
      digits = 2,
      escape = FALSE,
      booktabs = T,
      label = "mpfree",
      caption = "Impact of our algorithm as a preprocessing step for minimal presentations.
      Inside each group of columns, the ``Build (s)'' column displays the time taken in seconds to build the clique bifiltration, and ``mpfree (s)'' the time taken to run \\texttt{mpfree}.
      In addition, the ``Preprocessing (s)'' column displays the time taken to run our algorithm, and ``Speedup'' is the speedup compared to not doing preprocessing. $\\infty$ means
      that the algorithm ran out of memory during the execution.",
      col.names = c("Dataset",
                    "Build (s)", "mpfree (s)",
                    # "Removal (s)", "Build (s)", "mpfree (s)", "Speedup",
                    "Removal (s)", "Build (s)", "mpfree (s)", "Speedup"),
      align = c("l", rep("r", 6))) %>%
    kable_styling(latex_options = c("striped", "hold_position"), font_size = 8) %>%
    add_header_above(c(" " = 1,
                        "No preprocessing" = 2,
                       # "Filtration-domination" = 4,
                       "With our preprocessing" = 4)) %>%
    cat(., file = "compare_mpfree.tex")
}

do_multiple_iterations <- function() {
  multiple_iters_csv <- read.csv(file = "compare_multiple_iterations.csv") %>%
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
    relocate(c(Dataset, unlist(list.map(iters, iters ~ get_col(iters)))))

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
      col.names = c("Dataset", rep(c("Time (s)", "Removed"), 5))) %>%
    kable_styling(latex_options = c("striped", "scale_down", "hold_position")) %>%
    add_header_above(c(" " = 1,
                       "Iteration 1" = 2,
                       "Iteration 2" = 2,
                       "Iteration 3" = 2,
                       "Iteration 4" = 2,
                       "Iteration 5" = 2
                       )) %>%
    cat(., file = "compare_multiple_iterations.tex")
}

do_asymptotics <- function() {
  asymptotics_csv <- read.csv(file = "compare_asymptotics.csv") %>%
    mutate(Ratio = (After / Before) * 100) %>%
    mutate(Vertices = Points * Points)

  spheres <- asymptotics_csv %>%
    dplyr::filter(Dataset == "sphere") %>%
    group_by(Dataset, Points)
    # summarise(Ratio = mean(Ratio))
    # dplyr::filter(10. <= Ratio & Ratio <= 20.)

  asympt_width <- 4

  ggplot(spheres, aes(x = Before, y = Ratio)) +
    labs(x = "Edges", y = "Time (s)") +
    geom_point() +
    geom_smooth(method = "lm", formula = y ~ poly(x, 2), se = FALSE)

  ggsave("compare_asymptotics_sphere.pdf", width = asympt_width, height = asympt_width)

  ggplot(asymptotics_csv %>%
           dplyr::filter(Dataset == "torus" & Algorithm == "Strong filtration-domination"),
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Torus") +
    geom_smooth(method = "lm", formula = y ~ poly(x, 2), se = FALSE) +
    geom_point()

  ggsave("compare_asymptotics_torus.pdf", width = asympt_width, height = asympt_width)

  ggplot(asymptotics_csv %>%
           dplyr::filter(Dataset == "torus" & Algorithm == "Filtration-domination"),
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Torus") +
    geom_smooth(method = "lm", formula = y ~ poly(x, 3), se = FALSE) +
    geom_point()

    ggsave("compare_asymptotics_torus_full.pdf", width = asympt_width, height = asympt_width)

  ggplot(asymptotics_csv %>%
           dplyr::filter(Dataset == "uniform" & Algorithm == "Strong filtration-domination"), #%>%
           # dplyr::filter(Points == 200 | Points == 400 | Points == 800 | Points == 1600 | Points == 3000),
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Uniform") +
    geom_smooth(method = "lm",
                formula = y ~ poly(x, 2),
                se = FALSE) +
    geom_point()

  ggsave("compare_asymptotics_uniform.pdf", width = asympt_width, height = asympt_width)

  ggplot(asymptotics_csv %>%
           dplyr::filter(Dataset == "uniform" & Algorithm == "Filtration-domination"), #%>%
         # dplyr::filter(Points == 200 | Points == 400 | Points == 800 | Points == 1600 | Points == 3000),
         aes(x = Before, y = Time)) +
    labs(x = "Edges", y = "Time (s)", title = "Uniform") +
    geom_smooth(method = "lm",
                formula = y ~ poly(x, 2),
                se = FALSE) +
    geom_point()

  ggsave("compare_asymptotics_uniform_full.pdf", width = asympt_width, height = asympt_width)
}

args <- commandArgs(trailingOnly=TRUE)

if (length(args)==0) {
  stop("Give me a chart to do.", call.=FALSE)
}

command <- args[1]
if (command == "orders") {
  do_orders()
} else if (command == "removal") {
  do_removals()
} else if (command == "presentation") {
  do_removals_presentation()
  do_orders_presentation()
  do_mpfree_presentation()
} else if (command == "preliminary") {
  do_preliminary()
} else if (command == "mpfree") {
  do_mpfree()
} else if (command == "multiple_iterations") {
  do_multiple_iterations()
} else if (command == "asymptotics") {
  do_asymptotics()
} else if (command == "random-densities") {
  do_random_densities()
} else {
  stop("Unknown command.", call.= FALSE)
}
