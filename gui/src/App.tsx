// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useState, type ReactElement } from "react";
import {
  ActionIcon,
  AppShell,
  Badge,
  Box,
  Center,
  Group,
  Loader,
  NavLink,
  ScrollArea,
  Stack,
  Text,
  TextInput,
  Title,
  useMantineColorScheme,
} from "@mantine/core";
import {
  IconBrandGithub,
  IconMoon,
  IconSearch,
  IconSun,
} from "@tabler/icons-react";

import logoUrl from "/logo.svg";
import { listCalculators, type CalcSummary } from "./api/calc";
import { FeverPainCalculator } from "./calculators/FeverPain";

/**
 * Which calculators have a hand-crafted UI today. As we build more, we add
 * an entry here mapping the machine name to its React component. Anything
 * not in this set shows the "coming soon" placeholder when selected,
 * rather than a broken page.
 */
const IMPLEMENTED: Record<string, () => ReactElement> = {
  feverpain: FeverPainCalculator,
};

/**
 * The shortlist that sits at the top of the sidebar. Ordering reflects
 * the user's stated priorities for the first GUI cut: FeverPAIN as the
 * MVP, then CHA2DS2-VASc and QRISK3 (both politically high-impact - QRISK3
 * is still not implemented in EMIS or SystmOne).
 */
const FEATURED = ["feverpain", "cha2ds2vasc", "qrisk3"];

function ComingSoon({ clincalc }: { calc: CalcSummary }) {
  return (
    <Stack gap="md" maw={620}>
      <Title order={2}>{calc.title}</Title>
      <Text c="dimmed">{calc.description}</Text>
      <Box
        p="lg"
        style={{
          border: "1px dashed var(--mantine-color-default-border)",
          borderRadius: "var(--mantine-radius-md)",
        }}
      >
        <Text fw={600} mb={4}>
          GUI coming soon
        </Text>
        <Text size="sm" c="dimmed">
          The scoring logic for this calculator already ships in the{" "}
          <code>calc</code> CLI - the desktop UI is being built calculator by
          calculator, each one hand-crafted so the form fits the clinical
          question. Try it now with{" "}
          <code>calc {calc.name} --input examples/{calc.name}.json</code>.
        </Text>
      </Box>
    </Stack>
  );
}

function Brand() {
  return (
    <Group gap="sm" wrap="nowrap" style={{ minWidth: 0 }}>
      <img
        src={logoUrl}
        alt=""
        aria-hidden
        width={28}
        height={28}
        style={{ flexShrink: 0 }}
      />
      <Box style={{ minWidth: 0 }}>
        <Text className="brand-wordmark" fz="xl" c="teal">
          calc
        </Text>
        <Text size="xs" c="dimmed" lh={1}>
          open clinical calculators
        </Text>
      </Box>
    </Group>
  );
}

function ColorSchemeToggle() {
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const isDark = colorScheme === "dark";
  return (
    <ActionIcon
      variant="subtle"
      color="gray"
      onClick={() => toggleColorScheme()}
      title={isDark ? "Switch to light mode" : "Switch to dark mode"}
    >
      {isDark ? <IconSun size={18} /> : <IconMoon size={18} />}
    </ActionIcon>
  );
}

export default function App() {
  const [calcs, setCalcs] = useState<CalcSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string>("feverpain");
  const [query, setQuery] = useState("");

  useEffect(() => {
    listCalculators()
      .then(setCalcs)
      .catch((e: unknown) => setError(String(e)));
  }, []);

  const filtered = (calcs ?? []).filter((c) => {
    if (!query.trim()) return true;
    const q = query.toLowerCase();
    return (
      c.name.includes(q) ||
      c.title.toLowerCase().includes(q) ||
      c.tags.some((t) => t.includes(q))
    );
  });

  // Featured calcs at the top (in declared order); everything else
  // alphabetically below.
  const featured = filtered.filter((c) => FEATURED.includes(c.name));
  featured.sort(
    (a, b) => FEATURED.indexOf(a.name) - FEATURED.indexOf(b.name),
  );
  const others = filtered
    .filter((c) => !FEATURED.includes(c.name))
    .sort((a, b) => a.title.localeCompare(b.title));

  const current = calcs?.find((c) => c.name === selected) ?? null;
  const Body = current && IMPLEMENTED[current.name];

  return (
    <AppShell
      header={{ height: 56 }}
      navbar={{ width: 280, breakpoint: 0 }}
      padding="lg"
    >
      <AppShell.Header>
        <Group h="100%" px="lg" justify="space-between">
          <Brand />
          <Group gap="xs">
            <ColorSchemeToggle />
            <ActionIcon
              variant="subtle"
              color="gray"
              component="a"
              href="https://github.com/pacharanero/clincalc"
              target="_blank"
              rel="noreferrer"
              title="View on GitHub"
            >
              <IconBrandGithub size={18} />
            </ActionIcon>
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="sm">
        <Stack gap="xs" h="100%">
          <TextInput
            placeholder="Filter calculators…"
            leftSection={<IconSearch size={14} />}
            value={query}
            onChange={(e) => setQuery(e.currentTarget.value)}
            size="sm"
          />

          <ScrollArea style={{ flex: 1 }} type="hover">
            <Stack gap={2}>
              {!calcs && !error && (
                <Center py="xl">
                  <Loader size="xs" />
                </Center>
              )}
              {error && (
                <Text size="sm" c="red" p="sm">
                  {error}
                </Text>
              )}

              {featured.length > 0 && (
                <Text size="xs" tt="uppercase" c="dimmed" px="sm" pt={6}>
                  Featured
                </Text>
              )}
              {featured.map((c) => (
                <CalcLink
                  key={c.name}
                  calc={c}
                  active={selected === c.name}
                  implemented={c.name in IMPLEMENTED}
                  onClick={() => setSelected(c.name)}
                />
              ))}

              {others.length > 0 && (
                <Text size="xs" tt="uppercase" c="dimmed" px="sm" pt="md">
                  All calculators ({others.length})
                </Text>
              )}
              {others.map((c) => (
                <CalcLink
                  key={c.name}
                  calc={c}
                  active={selected === c.name}
                  implemented={c.name in IMPLEMENTED}
                  onClick={() => setSelected(c.name)}
                />
              ))}
            </Stack>
          </ScrollArea>

          <Text size="xs" c="dimmed" px="sm">
            {calcs?.length ?? 0} calculators - 1 hand-crafted UI so far
          </Text>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        {!current && (
          <Center h="60vh">
            <Loader />
          </Center>
        )}
        {current && Body && <Body />}
        {current && !Body && <ComingSoon calc={current} />}
      </AppShell.Main>
    </AppShell>
  );
}

function CalcLink({
  calc,
  active,
  implemented,
  onClick,
}: {
  calc: CalcSummary;
  active: boolean;
  implemented: boolean;
  onClick: () => void;
}) {
  return (
    <NavLink
      active={active}
      onClick={onClick}
      label={
        <Group justify="space-between" wrap="nowrap" gap="xs">
          <Text size="sm" truncate>
            {calc.title}
          </Text>
          {calc.proprietary && (
            <Badge size="xs" color="red" variant="light">
              stub
            </Badge>
          )}
          {!calc.proprietary && implemented && (
            <Badge size="xs" color="teal" variant="light">
              ready
            </Badge>
          )}
        </Group>
      }
      description={
        <Text size="xs" c="dimmed">
          {calc.name}
        </Text>
      }
    />
  );
}
