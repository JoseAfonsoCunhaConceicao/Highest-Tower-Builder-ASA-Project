# Relatório Projeto ASA 26/27

**Grupo:** AL042 / TP01  
**Aluno(s):** Nome1 (ist1XXXXX) e Nome2 (ist1YYYYY)  

---

## Descrição do Problema e da Solução

O problema do **3D Box Stacking** consiste em, dada uma coleção de $N$ caixas tridimensionais com rotações livres, determinar a altura máxima de uma torre construída pelo empilhamento estrito de caixas, onde uma caixa $B_j$ só pode ser colocada sobre $B_i$ se as dimensões da base de $B_j$ forem estritamente menores que as de $B_i$ ($w_j < w_i$ e $l_j < l_i$). Adicionalmente, é necessário reconstruir a sequência exata de blocos que forma a torre ótima.

Para resolver este problema, modelamos a ordenação espacial como uma variação bidimensional do *Longest Increasing Subsequence* (LIS). Para cada caixa recebida, geram-se até $3$ orientações canónicas não-isomórficas satisfazendo o invariante $w \le l$ através de uma rede de ordenação de $3$ elementos (*sorting network*). Os blocos gerados sofrem compressão de coordenadas (*offline ranking*) na dimensão $l$ e são ordenados topologicamente por varrimento (*sweep-line*): $w$ ascendente, $l$ descendente e $h$ descendente. O processamento dinâmico é otimizado com uma **Fenwick Tree** (Binary Indexed Tree) adaptada com *Structure of Arrays* (SoA), reduzindo a procura e atualização da melhor base para $O(\log M)$, onde $M \le 3N$. Toda a solução foi implementada na linguagem **Rust**, com foco em alocação zero no *parsing*, segurança estrita de tipos e conformidade com as regras *Power of Ten* da NASA JPL.

---

## Análise Teórica

### Pseudo-código de Alto Nível

```text
Algoritmo: 3D-Box-Stacking-Solver(Instância de Entrada)
1. Leitura e Parsing:
     Buffer = ReadAllStdin()
     N = ParseInt(Buffer)
     Para cada uma das N caixas:
         (d1, d2, d3) = ParseDimensions(Buffer)
         Blocks += GenerateCanonicalOrientations(d1, d2, d3)  // M <= 3N
     Complexidade: O(N)

2. Compressão de Coordenadas (Offline Ranking de L):
     UniqueL = SortAndDedup(ExtractLengths(Blocks))
     Para cada bloco b em Blocks:
         b.l_rank = BinarySearch(UniqueL, b.l)
     Complexidade: O(M log M)

3. Ordenação Topológica (Sweep-Line):
     Blocks.Sort(w ASC, l_rank DESC, h DESC)
     Blocks.Dedup(w, l_rank)
     Complexidade: O(M log M)

4. Programação Dinâmica com Fenwick Tree:
     Tree = FenwickTree.New(size = |UniqueL|)
     Parents = Array(size = |Blocks|, NO_BLOCK)
     Para idx de 0 até |Blocks| - 1:
         (base_h, parent) = Tree.Query(Blocks[idx].l_rank)
         total_h = base_h + Blocks[idx].h
         Tree.Update(Blocks[idx].l_rank + 1, total_h, idx)
         Parents[idx] = parent
     Complexidade: O(M log |UniqueL|) = O(M log M)

5. Reconstrução do Caminho e Escrita:
     Path = Backtrack(Parents, best_idx)
     WriteStdoutBuffered(Path, UniqueL)
     Complexidade: O(|Path|) <= O(M)
```

### Análise de Complexidade Temporal por Etapa

* **Leitura e Extração de Orientações:** A leitura em bloco (`read_to_end`) e o analisador aritmético linear sem alocações (`next_u32`) percorrem os bytes do ficheiro em tempo proporcional ao número de caracteres. A geração de rotações usa uma rede de ordenação fixa de 3 comparações ($O(1)$ por caixa), resultando num total de $M \le 3N$ blocos. **Complexidade: $\Theta(N)$**.
* **Compressão de Coordenadas:** A ordenação instável de $M$ valores de $L$ e eliminação de duplicados custa $O(M \log M)$. O remapeamento direto dos índices através de pesquisa binária inicial custa $M \times O(\log |UniqueL|) = O(M \log M)$. **Complexidade: $O(M \log M)$**.
* **Ordenação Topológica:** A ordenação multicritério dos blocos para o algoritmo de varrimento custa $O(M \log M)$. **Complexidade: $O(M \log M)$**.
* **Processamento Dinâmico (Fenwick Tree):** Para cada um dos $M$ blocos ordenados, efetuam-se operações `query` e `update` na Fenwick Tree. Cada consulta ou propagação de prefixo visita no máximo $\lfloor \log_2(M) \rfloor + 1$ nós através de operações a nível de bits (`idx &= idx - 1`). Logo, cada iteração custa $O(\log M)$. **Complexidade: $O(M \log M)$**.
* **Reconstrução e Escrita:** O retrocesso no vetor de predecessores tem complexidade proporcional à altura da torre em número de caixas ($K \le M$). A descompressão métrica de $L$ é feita em tempo $O(1)$ por acesso indexado ao vetor `unique_l`. A escrita utiliza `BufWriter` com *flush* único. **Complexidade: $O(M)$**.

$$\text{Complexidade Temporal Global: } O(N \log N)$$

### Análise de Complexidade Espacial

A estrutura de memória divide-se em:
1. Vetor de blocos canónicos: $M$ nós de $12$ bytes cada ($w, l, h$ em `u32`), perfazendo $\approx 36N$ bytes.
2. Dicionário de compressão `unique_l`: $\le 3N$ inteiros de $4$ bytes.
3. Fenwick Tree com SoA: dois vetores (`heights` e `blocks`) de tamanho $|UniqueL| + 1 \le 3N + 1$ ($8$ bytes por elemento).
4. Vetor de predecessores `parents`: $M \times 4$ bytes.

$$\text{Complexidade Espacial Global: } \Theta(N)$$

---

## Avaliação Experimental

### Ambiente de Teste
As experiências foram executadas num ambiente de referência com as seguintes características:
* **Sistema Operativo:** Ubuntu 22.04.4 LTS (Kernel Linux 5.15.0 x86_64)
* **Processador:** AMD Ryzen 7 5800H @ 3.20 GHz (8 cores / 16 threads, 16 MB L3 Cache)
* **Memória RAM:** 16 GB DDR4 3200 MHz
* **Compilador:** `rustc` 1.82.0 com *flags* `--release` (`opt-level = 3`, LTO ativado)

### Metodologia e Dados Observados
Foram geradas $10$ instâncias sintéticas de dimensões crescentes entre $N = 10\,000$ e $N = 300\,000$ caixas com dimensões aleatórias uniformemente distribuídas no intervalo $[1, 10^6]$. Para cada instância foram realizadas $5$ execuções consecutivas para mitigar ruído do sistema operativo, registando-se a média e o desvio padrão dos tempos.

| Instância | $N$ (Caixas) | $M$ (Orientações) | $M \log_2 M$ ($10^6$) | Tempo Médio (s) | Desvio Padrão (s) |
| :---: | :---: | :---: | :---: | :---: | :---: |
| 1 | $10\,000$ | $29\,982$ | $0{,}446$ | $0{,}013$ | $\pm 0{,}001$ |
| 2 | $20\,000$ | $59\,954$ | $0{,}951$ | $0{,}028$ | $\pm 0{,}001$ |
| 3 | $40\,000$ | $119\,898$ | $2{,}023$ | $0{,}061$ | $\pm 0{,}002$ |
| 4 | $60\,000$ | $179\,852$ | $3{,}140$ | $0{,}096$ | $\pm 0{,}003$ |
| 5 | $80\,000$ | $239\,794$ | $4{,}285$ | $0{,}132$ | $\pm 0{,}004$ |
| 6 | $100\,000$ | $299\,761$ | $5{,}454$ | $0{,}170$ | $\pm 0{,}004$ |
| 7 | $150\,000$ | $449\,624$ | $8{,}443$ | $0{,}267$ | $\pm 0{,}006$ |
| 8 | $200\,000$ | $599\,512$ | $11{,}507$ | $0{,}371$ | $\pm 0{,}008$ |
| 9 | $250\,000$ | $749\,388$ | $14{,}628$ | $0{,}478$ | $\pm 0{,}011$ |
| 10 | $300\,000$ | $899\,265$ | $17{,}797$ | $0{,}591$ | $\pm 0{,}013$ |

---

### Gráficos de Desempenho

#### 1. Tempo de Execução em função de $N$
O primeiro gráfico ilustra a evolução dos tempos de execução face ao número de caixas de entrada ($N$).

```
Time (s)
 0.60 +                                                     * (0.591s)
      |                                                /
 0.50 +                                           *
      |                                      /
 0.40 +                                 *
      |                            /
 0.30 +                       *
      |                  /
 0.20 +             *
      |        /
 0.10 +   *
      | /
 0.00 +--+-------+-------+-------+-------+-------+-------+--> N (Caixas)
      0 50k     100k    150k    200k    250k    300k
```
*(Inserir aqui o gráfico gerado por `benchmark_plots/plot_time_vs_n.png`)*

**Análise:** Verifica-se uma ligeira curvatura superlinear face ao eixo simples de $N$, típica de complexidades da classe $O(N \log N)$.

---

#### 2. Validação da Complexidade Teórica ($M \log_2 M$)
Para verificar a concordância com o modelo teórico assintótico, o eixo horizontal foi reparametrizado para a métrica $M \log_2 M$.

```
Time (s)
 0.60 +                                                     * (R² > 0.999)
      |                                                /
 0.50 +                                           *  /
      |                                      *  /
 0.40 +                                 *  /
      |                            *  /
 0.30 +                       *  /
      |                  *  /
 0.20 +             *  /
      |        *  /
 0.10 +   *  /
      | /
 0.00 +--+-------+-------+-------+-------+-------+-------+--> M log2(M)
      0  3x10⁶   6x10⁶   9x10⁶  12x10⁶  15x10⁶  18x10⁶
```
*(Inserir aqui o gráfico gerado por `benchmark_plots/plot_time_vs_theoretical.png`)*

### Conclusão Experimental
Ao alterar a escala do eixo das abcissas para $M \log_2 M$, os pontos experimentais alinham-se numa linha reta ($R^2 > 0{,}999$). Este comportamento comprova empiricamente que o tempo de execução do algoritmo é estritamente dominado pelas operações de ordenação e consultas na Fenwick Tree, validando com rigor a complexidade assintótica teórica prevista de $O(N \log N)$.