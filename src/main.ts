import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Box,
  BookOpen,
  Check,
  ChevronDown,
  CircleAlert,
  CircleCheck,
  Command,
  createIcons,
  Download,
  FolderOpen,
  FolderPlus,
  MoreVertical,
  PackagePlus,
  Plus,
  RefreshCw,
  Search,
  Terminal,
  Trash2,
  Users,
  X,
} from "lucide";
import "./styles.css";

type SourceType = "local" | "command";
type View = "skills" | "groups" | "install" | "docs";
type InstallMode = "skill" | "group";

type Skill = {
  id: string;
  name: string;
  description: string;
  version?: string | null;
  sourceType: SourceType;
  libraryPath?: string | null;
  installCommand?: string | null;
  tags: string[];
};

type SkillGroup = {
  id: string;
  name: string;
  description?: string | null;
  skillIds: string[];
};

type Catalog = {
  schemaVersion: number;
  skills: Skill[];
  groups: SkillGroup[];
};

type InstallCommandPreview = {
  skillId: string;
  skillName: string;
  command: string;
};

type InstallationOutcome = {
  targetDir: string;
  installedSkills: Array<{
    skillId: string;
    skillName: string;
    targetPath?: string | null;
    command?: string | null;
    executed: boolean;
  }>;
  commandPreviews: InstallCommandPreview[];
};

type TauriWindow = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: unknown;
};

const app = document.querySelector<HTMLDivElement>("#app")!;
const iconSet = {
  Box,
  BookOpen,
  Check,
  ChevronDown,
  CircleAlert,
  CircleCheck,
  Command,
  Download,
  FolderOpen,
  FolderPlus,
  MoreVertical,
  PackagePlus,
  Plus,
  RefreshCw,
  Search,
  Terminal,
  Trash2,
  Users,
  X,
};

const demoCatalog: Catalog = {
  schemaVersion: 1,
  skills: [
    {
      id: "demo/local-skill",
      name: "Local skill",
      description: "Importe un dossier contenant un SKILL.md.",
      version: "demo",
      sourceType: "local",
      libraryPath: "C:/path/to/skill",
      installCommand: null,
      tags: ["local", "codex", "workflow"],
    },
    {
      id: "demo/command-skill",
      name: "Command skill",
      description: "Reference une commande d'installation externe.",
      version: "demo",
      sourceType: "command",
      libraryPath: null,
      installCommand: "codex skill install example",
      tags: ["cli", "install"],
    },
  ],
  groups: [
    {
      id: "demo/starter-pack",
      name: "Starter pack",
      description: "Installation rapide de plusieurs skills.",
      skillIds: ["demo/local-skill", "demo/command-skill"],
    },
  ],
};

const state = {
  catalog: { schemaVersion: 1, skills: [], groups: [] } as Catalog,
  view: "skills" as View,
  query: "",
  selectedSkillId: "",
  selectedGroupId: "",
  installMode: "skill" as InstallMode,
  installReference: "",
  installProject: "",
  installTarget: "",
  overwrite: false,
  actionsOpen: false,
  commandModalOpen: false,
  groupModalOpen: false,
  notice: "",
  error: "",
  loading: false,
};

function isTauriRuntime(): boolean {
  const tauriWindow = window as TauriWindow;
  return Boolean(tauriWindow.__TAURI__ || tauriWindow.__TAURI_INTERNALS__);
}

function escapeHtml(value: unknown): string {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function filteredSkills(): Skill[] {
  const query = state.query.trim().toLowerCase();
  if (!query) return state.catalog.skills;
  return state.catalog.skills.filter((skill) => {
    return (
      skill.name.toLowerCase().includes(query) ||
      skill.id.toLowerCase().includes(query) ||
      skill.description.toLowerCase().includes(query) ||
      skill.tags.some((tag) => tag.toLowerCase().includes(query))
    );
  });
}

function selectedSkill(): Skill | undefined {
  return state.catalog.skills.find((skill) => skill.id === state.selectedSkillId);
}

function selectedGroup(): SkillGroup | undefined {
  return state.catalog.groups.find((group) => group.id === state.selectedGroupId);
}

function skillName(id: string): string {
  return state.catalog.skills.find((skill) => skill.id === id)?.name ?? id;
}

async function refreshCatalog() {
  if (isTauriRuntime()) {
    state.catalog = await invoke<Catalog>("get_catalog");
  } else {
    state.catalog = demoCatalog;
    if (!state.notice) {
      state.notice = "Apercu web: les actions systeme sont dans l'app Tauri.";
    }
  }

  if (!state.catalog.skills.some((skill) => skill.id === state.selectedSkillId)) {
    state.selectedSkillId = "";
  }
  if (!state.catalog.groups.some((group) => group.id === state.selectedGroupId)) {
    state.selectedGroupId = "";
  }
  if (!state.installReference) {
    state.installReference =
      state.installMode === "skill"
        ? state.catalog.skills[0]?.id ?? ""
        : state.catalog.groups[0]?.id ?? "";
  }
}

async function withTask(task: () => Promise<void>) {
  state.loading = true;
  state.error = "";
  state.notice = "";
  state.actionsOpen = false;
  render();
  try {
    await task();
  } catch (error) {
    state.error = String(error);
  } finally {
    state.loading = false;
    render();
  }
}

async function chooseDirectory(): Promise<string | null> {
  if (!isTauriRuntime()) {
    state.notice = "Selection de dossier disponible dans la fenetre Tauri.";
    return null;
  }
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

function render() {
  const skills = filteredSkills();
  const skill = selectedSkill();
  const group = selectedGroup();
  const panelOpen = (state.view === "skills" && skill) || (state.view === "groups" && group);

  app.innerHTML = `
    <div class="shell">
      <aside class="sidebar">
        <div class="brand">
          <span class="brand-mark">S</span>
          <strong>Skills</strong>
        </div>
        <nav class="nav">
          ${navButton("skills", "box", "Skills", String(state.catalog.skills.length))}
          ${navButton("groups", "users", "Groupes", String(state.catalog.groups.length))}
          ${navButton("docs", "book-open", "Docs CLI")}
        </nav>
      </aside>

      <main class="workspace">
        <header class="topbar">
          <div class="title-block">
            <h1>${pageTitle(skills.length)}</h1>
            <span>${pageSubtitle(skills.length)}</span>
          </div>

          <div class="topbar-controls">
            ${state.view === "skills" || state.view === "groups" ? renderSearch() : ""}
            <div class="actions-menu">
              <button class="button primary" id="actions-toggle" type="button">
                Actions
                <i data-lucide="chevron-down"></i>
              </button>
              ${state.actionsOpen ? renderActionsDropdown() : ""}
            </div>
          </div>
        </header>

        ${statusBar()}

        <section class="content ${panelOpen ? "has-panel" : ""}">
          ${state.view === "skills" ? renderSkillsView(skills, skill) : ""}
          ${state.view === "groups" ? renderGroupsView(group) : ""}
          ${state.view === "install" ? renderInstallView() : ""}
          ${state.view === "docs" ? renderDocsView() : ""}
        </section>
      </main>
    </div>

    ${state.commandModalOpen ? renderCommandModal() : ""}
    ${state.groupModalOpen ? renderGroupModal() : ""}
  `;

  bindEvents();
  createIcons({ icons: iconSet });
}

function pageTitle(visibleSkills: number): string {
  if (state.view === "skills") return "Skills";
  if (state.view === "groups") return "Groupes";
  if (state.view === "install") return "Installer";
  return "Docs CLI";
}

function pageSubtitle(visibleSkills: number): string {
  if (state.view === "skills") return `${visibleSkills} visibles`;
  if (state.view === "groups") return `${state.catalog.groups.length} groupes`;
  if (state.view === "install") return "Installation";
  return "Reference exacte du binaire skills-list";
}

function renderSearch(): string {
  return `
    <div class="search">
      <i data-lucide="search"></i>
      <input id="search" placeholder="Rechercher" value="${escapeHtml(state.query)}" />
    </div>
  `;
}

function navButton(view: View, icon: string, label: string, badge = ""): string {
  const selected = state.view === view ? "selected" : "";
  return `
    <button class="nav-button ${selected}" data-view="${view}">
      <i data-lucide="${icon}"></i>
      <span>${label}</span>
      ${badge ? `<b>${escapeHtml(badge)}</b>` : ""}
    </button>
  `;
}

function renderActionsDropdown(): string {
  return `
    <div class="dropdown">
      <button type="button" id="open-command-modal">
        <i data-lucide="command"></i>
        Ajouter une commande
      </button>
      <button type="button" id="open-group-modal">
        <i data-lucide="users"></i>
        Creer un groupe
      </button>
      <button type="button" id="import-skill">
        <i data-lucide="folder-plus"></i>
        Importer
      </button>
      <button type="button" id="go-install">
        <i data-lucide="package-plus"></i>
        Installer
      </button>
    </div>
  `;
}

function statusBar(): string {
  if (state.loading) {
    return `<div class="status muted"><i data-lucide="refresh-cw"></i> Synchronisation...</div>`;
  }
  if (state.error) {
    return `
      <div class="status error">
        <span><i data-lucide="circle-alert"></i> ${escapeHtml(state.error)}</span>
        <button class="status-close" id="dismiss-status" type="button" title="Fermer le message">
          <i data-lucide="x"></i>
        </button>
      </div>
    `;
  }
  if (state.notice) {
    return `
      <div class="status success">
        <span><i data-lucide="circle-check"></i> ${escapeHtml(state.notice)}</span>
        <button class="status-close" id="dismiss-status" type="button" title="Fermer le message">
          <i data-lucide="x"></i>
        </button>
      </div>
    `;
  }
  return "";
}

function renderSkillsView(skills: Skill[], skill?: Skill): string {
  return `
    <div class="main-column">
      <div class="list">
        ${
          skills.length
            ? skills.map(renderSkillRow).join("")
            : renderEmptyState("Aucun skill", "Importe un dossier SKILL.md ou ajoute une commande.")
        }
      </div>
    </div>
    ${skill ? `<aside class="panel">${renderSkillPanel(skill)}</aside>` : ""}
  `;
}

function renderSkillRow(skill: Skill): string {
  const selected = skill.id === state.selectedSkillId ? "selected" : "";
  return `
    <button class="row ${selected}" data-skill="${escapeHtml(skill.id)}">
      <span class="row-icon"><i data-lucide="${skill.sourceType === "command" ? "terminal" : "box"}"></i></span>
      <span class="row-main">
        <strong>${escapeHtml(skill.name)}</strong>
        <small>${escapeHtml(skill.id)}</small>
      </span>
      <span class="row-kind">${escapeHtml(skill.sourceType)}</span>
    </button>
  `;
}

function renderSkillPanel(skill: Skill): string {
  return `
    <div class="panel-head">
      <div>
        <h2>${escapeHtml(skill.name)}</h2>
        <p>${escapeHtml(skill.id)}</p>
      </div>
      <button class="icon-button" id="close-panel" type="button" title="Fermer">
        <i data-lucide="x"></i>
      </button>
    </div>
    <p class="description">${escapeHtml(skill.description || "Sans description")}</p>
    <dl class="details">
      <dt>Source</dt>
      <dd>${escapeHtml(skill.sourceType)}</dd>
      <dt>Version</dt>
      <dd>${escapeHtml(skill.version || "Non definie")}</dd>
      <dt>Tags</dt>
      <dd>${skill.tags.length ? skill.tags.map((tag) => `<span class="tag">${escapeHtml(tag)}</span>`).join("") : "Aucun"}</dd>
      ${
        skill.installCommand
          ? `<dt>Commande</dt><dd><code>${escapeHtml(skill.installCommand)}</code></dd>`
          : `<dt>Dossier</dt><dd><code>${escapeHtml(skill.libraryPath || "Bibliotheque locale")}</code></dd>`
      }
    </dl>
    <div class="panel-actions">
      <button class="button primary" id="install-selected"><i data-lucide="package-plus"></i>Installer</button>
      <button class="button secondary" id="export-selected" ${skill.sourceType === "command" ? "disabled" : ""}><i data-lucide="download"></i>Exporter</button>
      <button class="button danger" id="delete-selected"><i data-lucide="trash-2"></i>Supprimer</button>
    </div>
  `;
}

function renderGroupsView(group?: SkillGroup): string {
  return `
    <div class="main-column">
      <div class="list">
        ${
          state.catalog.groups.length
            ? state.catalog.groups.map(renderGroupRow).join("")
            : renderEmptyState("Aucun groupe", "Cree un groupe pour combiner plusieurs skills.")
        }
      </div>
    </div>
    ${group ? `<aside class="panel">${renderGroupPanel(group)}</aside>` : ""}
  `;
}

function renderGroupRow(group: SkillGroup): string {
  const selected = group.id === state.selectedGroupId ? "selected" : "";
  return `
    <button class="row ${selected}" data-group="${escapeHtml(group.id)}">
      <span class="row-icon"><i data-lucide="users"></i></span>
      <span class="row-main">
        <strong>${escapeHtml(group.name)}</strong>
        <small>${escapeHtml(group.description || group.id)}</small>
      </span>
      <span class="row-kind">${group.skillIds.length}</span>
    </button>
  `;
}

function renderGroupPanel(group: SkillGroup): string {
  const members = new Set(group.skillIds);
  return `
    <div class="panel-head">
      <div>
        <h2>${escapeHtml(group.name)}</h2>
        <p>${group.skillIds.length} skill(s)</p>
      </div>
      <button class="icon-button" id="close-panel" type="button" title="Fermer">
        <i data-lucide="x"></i>
      </button>
    </div>
    <div class="member-list">
      ${
        state.catalog.skills.length
          ? state.catalog.skills
              .map((skill) => {
                const included = members.has(skill.id);
                return `
                  <button class="member-row ${included ? "included" : ""}" data-toggle-skill="${escapeHtml(skill.id)}">
                    <span>
                      <strong>${escapeHtml(skill.name)}</strong>
                      <small>${included ? "Dans le groupe" : "Disponible"}</small>
                    </span>
                    <i data-lucide="${included ? "check" : "plus"}"></i>
                  </button>
                `;
              })
              .join("")
          : renderEmptyState("Aucun skill", "Importe des skills avant de composer un groupe.")
      }
    </div>
    <div class="panel-actions">
      <button class="button primary" id="install-group"><i data-lucide="package-plus"></i>Installer</button>
      <button class="button danger" id="delete-group"><i data-lucide="trash-2"></i>Supprimer</button>
    </div>
  `;
}

function renderInstallView(): string {
  const references =
    state.installMode === "skill" ? state.catalog.skills : state.catalog.groups;
  if (!references.some((item) => item.id === state.installReference)) {
    state.installReference = references[0]?.id ?? "";
  }

  return `
    <div class="install-page">
      <div class="segmented">
        <button class="${state.installMode === "skill" ? "selected" : ""}" data-install-mode="skill">Skill</button>
        <button class="${state.installMode === "group" ? "selected" : ""}" data-install-mode="group">Groupe</button>
      </div>
      <form class="form-card" id="install-form">
        <label>
          Reference
          <select name="reference" required>
            ${references
              .map(
                (item) =>
                  `<option value="${escapeHtml(item.id)}" ${item.id === state.installReference ? "selected" : ""}>${escapeHtml(item.name)}</option>`,
              )
              .join("")}
          </select>
        </label>
        <label>
          Projet
          <div class="path-row">
            <input name="project" value="${escapeHtml(state.installProject)}" placeholder="Dossier du projet" />
            <button class="icon-button" type="button" id="choose-project" title="Choisir"><i data-lucide="folder-open"></i></button>
          </div>
        </label>
        <label>
          Cible optionnelle
          <div class="path-row">
            <input name="target" value="${escapeHtml(state.installTarget)}" placeholder="Remplace .agents/skills" />
            <button class="icon-button" type="button" id="choose-target" title="Choisir"><i data-lucide="folder-open"></i></button>
          </div>
        </label>
        <label class="check-row">
          <input type="checkbox" name="overwrite" ${state.overwrite ? "checked" : ""} />
          Ecraser si la cible existe
        </label>
        <button class="button primary" type="submit"><i data-lucide="package-plus"></i>Installer</button>
      </form>
    </div>
  `;
}

function renderDocsView(): string {
  return `
    <div class="docs-page">
      <section class="docs-card">
        <h2>CLI</h2>
        <p>Le binaire s'appelle <code>skills-list</code>. En developpement, lance-le avec <code>cargo run -p skills-cli --</code>.</p>
        <pre><code>skills-list [--data-dir &lt;DIR&gt;] &lt;COMMAND&gt;</code></pre>
      </section>

      <section class="docs-grid">
        ${docCommand(
          "search",
          "Recherche dans le catalogue par nom, id, description ou tag.",
          "skills-list search <query> [--json]",
          [
            "cargo run -p skills-cli -- search tauri",
            "cargo run -p skills-cli -- search tauri --json",
          ],
        )}
        ${docCommand(
          "import",
          "Importe un dossier contenant un SKILL.md, ou un dossier parent contenant plusieurs skills.",
          "skills-list import <path>",
          ["cargo run -p skills-cli -- import C:\\\\path\\\\to\\\\skill-or-skills-folder"],
        )}
        ${docCommand(
          "add-command",
          "Ajoute un skill qui execute une commande au lieu de copier un dossier local.",
          "skills-list add-command <name> --description <text> --command <cmd> [--tag <tag> ...]",
          [
            'cargo run -p skills-cli -- add-command "Team Bootstrap" --description "Install shared skills" --command "codex skill install team-bootstrap"',
            'cargo run -p skills-cli -- add-command "Echo Skill" --description "Runs echo" --command "echo ok" --tag cli --tag test',
          ],
        )}
        ${docCommand(
          "group create",
          "Cree un groupe vide. L'id est genere depuis le nom.",
          "skills-list group create <name> [--description <text>]",
          [
            "cargo run -p skills-cli -- group create Starter",
            'cargo run -p skills-cli -- group create Starter --description "Base project skills"',
          ],
        )}
        ${docCommand(
          "group add",
          "Ajoute un skill dans un groupe. Les references acceptent l'id ou le nom.",
          "skills-list group add <group> <skill>",
          ["cargo run -p skills-cli -- group add starter tauri-v2"],
        )}
        ${docCommand(
          "group remove",
          "Retire un skill d'un groupe.",
          "skills-list group remove <group> <skill>",
          ["cargo run -p skills-cli -- group remove starter tauri-v2"],
        )}
        ${docCommand(
          "group delete",
          "Supprime un groupe du catalogue.",
          "skills-list group delete <group>",
          ["cargo run -p skills-cli -- group delete starter"],
        )}
        ${docCommand(
          "install skill",
          "Installe un skill local ou commande. Par defaut la cible est <project>/.agents/skills.",
          "skills-list install skill <reference> [--project <DIR>] [--target <DIR>] [--overwrite] [--yes]",
          [
            "cargo run -p skills-cli -- install skill tauri-v2 --project C:\\\\path\\\\to\\\\project",
            "cargo run -p skills-cli -- install skill echo-skill --project C:\\\\path\\\\to\\\\project --yes",
          ],
        )}
        ${docCommand(
          "install group",
          "Installe tous les skills d'un groupe dans la meme cible.",
          "skills-list install group <reference> [--project <DIR>] [--target <DIR>] [--overwrite] [--yes]",
          ["cargo run -p skills-cli -- install group starter --project C:\\\\path\\\\to\\\\project --overwrite"],
        )}
      </section>

      <section class="docs-card">
        <h2>Tags, recherche et format SKILL.md</h2>
        <p>Les tags sont des mots-clefs de classement. Ils aident surtout a retrouver un skill avec <code>search</code>; ils ne changent pas le comportement d'installation.</p>
        <div class="docs-columns">
          <div>
            <h3>Skill local importe</h3>
            <p>Pour un dossier local, les tags viennent du frontmatter YAML de <code>SKILL.md</code>. Les champs reconnus sont <code>name</code>, <code>description</code>, <code>version</code> et <code>tags</code>.</p>
            <pre><code>---
name: Tauri v2
description: Developpement app Tauri
version: 1.0.0
tags:
  - tauri
  - rust
  - desktop
---
# Tauri v2</code></pre>
          </div>
          <div>
            <h3>Skill commande</h3>
            <p>Pour un skill cree par le CLI, ajoute un tag avec <code>--tag</code>. L'option est repetable: un tag par occurrence.</p>
            <pre><code>skills-list add-command "Setup Team" --description "Installe les skills communs" --command "codex skill install team" --tag cli --tag onboarding</code></pre>
          </div>
        </div>
        <dl class="docs-list">
          <dt>Recherche par tag</dt>
          <dd><code>skills-list search cli</code> trouve les tags contenant <code>cli</code>, sans tenir compte de la casse. La recherche regarde aussi le nom, l'id et la description.</dd>
          <dt>Nom des tags</dt>
          <dd>Utilise des tags courts et stables, par exemple <code>cli</code>, <code>tauri</code>, <code>rust</code>, <code>frontend</code>, <code>agent</code> ou <code>onboarding</code>. Le CLI ne demande pas de prefixe <code>#</code>.</dd>
          <dt>Difference avec les groupes</dt>
          <dd>Un tag classe un skill. Un groupe installe une liste precise de skills. Pour installer plusieurs elements ensemble, cree un groupe puis ajoute les skills voulus.</dd>
        </dl>
      </section>

      <section class="docs-card">
        <h2>Besoins par commande</h2>
        <p>Chaque commande a un minimum obligatoire. Les options entre crochets sont facultatives.</p>
        <dl class="docs-list dense">
          <dt><code>search &lt;query&gt; [--json]</code></dt>
          <dd><code>&lt;query&gt;</code> est requis. <code>--json</code> renvoie les skills complets en JSON formate.</dd>
          <dt><code>import &lt;path&gt;</code></dt>
          <dd><code>&lt;path&gt;</code> doit pointer vers un dossier avec un <code>SKILL.md</code>, ou vers un dossier parent contenant plusieurs sous-dossiers de skills.</dd>
          <dt><code>add-command &lt;name&gt; --description &lt;text&gt; --command &lt;cmd&gt;</code></dt>
          <dd><code>--description</code> et <code>--command</code> sont obligatoires. Les tags sont optionnels avec <code>--tag &lt;tag&gt;</code> repete autant de fois que necessaire.</dd>
          <dt><code>group create &lt;name&gt; [--description &lt;text&gt;]</code></dt>
          <dd><code>&lt;name&gt;</code> est obligatoire. L'id du groupe est genere depuis ce nom.</dd>
          <dt><code>group add &lt;group&gt; &lt;skill&gt;</code> / <code>group remove &lt;group&gt; &lt;skill&gt;</code></dt>
          <dd><code>&lt;group&gt;</code> et <code>&lt;skill&gt;</code> acceptent un id ou un nom exact, sans tenir compte de la casse.</dd>
          <dt><code>install skill|group &lt;reference&gt;</code></dt>
          <dd><code>&lt;reference&gt;</code> est obligatoire. Ajoute <code>--project</code> pour choisir le projet, <code>--target</code> pour forcer le dossier final, <code>--overwrite</code> pour remplacer, et <code>--yes</code> pour executer les command skills sans confirmation.</dd>
        </dl>
      </section>

      <section class="docs-card">
        <h2>Options globales et donnees</h2>
        <dl class="docs-list">
          <dt><code>--data-dir &lt;DIR&gt;</code></dt>
          <dd>Change le dossier de donnees du catalogue pour cette commande.</dd>
          <dt><code>SKILLS_LIST_DATA_DIR</code></dt>
          <dd>Variable d'environnement equivalente a <code>--data-dir</code>.</dd>
          <dt>References</dt>
          <dd>Les skills et groupes peuvent etre references par id, ou par nom exact sans tenir compte de la casse.</dd>
          <dt>Generation des ids</dt>
          <dd>Les ids sont crees depuis le nom: lettres/chiffres en minuscules, autres caracteres remplaces par <code>-</code>. Si un id existe deja, le CLI ajoute un suffixe comme <code>-2</code>.</dd>
          <dt>Cible d'installation</dt>
          <dd><code>--target</code> remplace la cible. Sinon le CLI utilise <code>--project/.agents/skills</code>, ou le dossier courant si <code>--project</code> est absent.</dd>
          <dt>Overwrite</dt>
          <dd>Si une cible existe deja, l'installation echoue sans <code>--overwrite</code>.</dd>
          <dt>Command skills</dt>
          <dd>Le CLI affiche les commandes avant execution. <code>--yes</code> execute sans prompt interactif.</dd>
        </dl>
      </section>

      <section class="docs-card">
        <h2>Sorties exactes utiles</h2>
        <pre><code>Imported &lt;name&gt; (&lt;id&gt;)
Added command skill &lt;name&gt; (&lt;id&gt;)
Created group &lt;name&gt; (&lt;id&gt;)
Added &lt;skill&gt; to group &lt;group-name&gt;
Removed &lt;skill&gt; from group &lt;group-name&gt;
Deleted group &lt;name&gt; (&lt;id&gt;)
Target: &lt;target-dir&gt;
Installed &lt;skill-name&gt; -&gt; &lt;target-path&gt;
Executed command skill &lt;skill-name&gt;
Pending command for &lt;skill-name&gt;: &lt;command&gt;</code></pre>
      </section>
    </div>
  `;
}

function docCommand(title: string, description: string, usage: string, examples: string[]): string {
  return `
    <article class="doc-command">
      <h3>${escapeHtml(title)}</h3>
      <p>${escapeHtml(description)}</p>
      <pre><code>${escapeHtml(usage)}</code></pre>
      <div class="examples">
        ${examples.map((example) => `<code>${escapeHtml(example)}</code>`).join("")}
      </div>
    </article>
  `;
}

function renderCommandModal(): string {
  return `
    <div class="modal-backdrop">
      <form class="modal" id="add-command-form">
        <div class="modal-head">
          <div>
            <h2>Ajouter une commande</h2>
            <p>Reference un installateur externe.</p>
          </div>
          <button class="icon-button" id="close-command-modal" type="button" title="Fermer"><i data-lucide="x"></i></button>
        </div>
        <label>
          Nom
          <input name="name" required placeholder="Nom du skill" />
        </label>
        <label>
          Description
          <input name="description" required placeholder="Description courte" />
        </label>
        <label>
          Commande
          <textarea name="command" required placeholder="codex skill install ..."></textarea>
        </label>
        <label>
          Tags
          <input name="tags" placeholder="dev, cli, agent" />
        </label>
        <button class="button primary" type="submit"><i data-lucide="terminal"></i>Ajouter</button>
      </form>
    </div>
  `;
}

function renderGroupModal(): string {
  return `
    <div class="modal-backdrop">
      <form class="modal" id="create-group-form">
        <div class="modal-head">
          <div>
            <h2>Creer un groupe</h2>
            <p>Regroupe plusieurs skills pour les installer ensemble.</p>
          </div>
          <button class="icon-button" id="close-group-modal" type="button" title="Fermer"><i data-lucide="x"></i></button>
        </div>
        <label>
          Nom
          <input name="name" required placeholder="Nom du groupe" />
        </label>
        <label>
          Description
          <input name="description" placeholder="Description courte" />
        </label>
        <button class="button primary" type="submit"><i data-lucide="users"></i>Creer</button>
      </form>
    </div>
  `;
}

function renderEmptyState(title: string, body: string): string {
  return `
    <div class="empty">
      <strong>${escapeHtml(title)}</strong>
      <p>${escapeHtml(body)}</p>
    </div>
  `;
}

function bindEvents() {
  app.onclick = (event) => {
    const target = event.target as Element | null;
    if (state.actionsOpen && !target?.closest(".actions-menu")) {
      state.actionsOpen = false;
      render();
    }
  };

  document.querySelectorAll<HTMLButtonElement>("[data-view]").forEach((button) => {
    button.addEventListener("click", () => {
      state.view = button.dataset.view as View;
      state.selectedSkillId = "";
      state.selectedGroupId = "";
      state.actionsOpen = false;
      render();
    });
  });

  document.querySelector<HTMLInputElement>("#search")?.addEventListener("input", (event) => {
    state.query = (event.target as HTMLInputElement).value;
    state.actionsOpen = false;
    render();
  });

  document.querySelector<HTMLButtonElement>("#actions-toggle")?.addEventListener("click", () => {
    state.actionsOpen = !state.actionsOpen;
    render();
  });

  document.querySelector<HTMLButtonElement>("#dismiss-status")?.addEventListener("click", () => {
    state.notice = "";
    state.error = "";
    render();
  });

  document.querySelector<HTMLButtonElement>("#open-command-modal")?.addEventListener("click", () => {
    state.actionsOpen = false;
    state.commandModalOpen = true;
    render();
  });

  document.querySelector<HTMLButtonElement>("#open-group-modal")?.addEventListener("click", () => {
    state.actionsOpen = false;
    state.groupModalOpen = true;
    render();
  });

  document.querySelector<HTMLButtonElement>("#close-command-modal")?.addEventListener("click", () => {
    state.commandModalOpen = false;
    render();
  });

  document.querySelector<HTMLButtonElement>("#close-group-modal")?.addEventListener("click", () => {
    state.groupModalOpen = false;
    render();
  });

  document.querySelector<HTMLButtonElement>("#go-install")?.addEventListener("click", () => {
    state.view = "install";
    state.actionsOpen = false;
    state.commandModalOpen = false;
    state.groupModalOpen = false;
    state.selectedSkillId = "";
    state.selectedGroupId = "";
    state.installMode = "skill";
    state.installReference = state.catalog.skills[0]?.id || "";
    render();
  });

  document.querySelector<HTMLButtonElement>("#import-skill")?.addEventListener("click", () => {
    withTask(async () => {
      const path = await chooseDirectory();
      if (!path) return;
      const imported = await invoke<Skill[]>("import_path", { path });
      await refreshCatalog();
      state.selectedSkillId = imported[0]?.id ?? "";
      state.view = "skills";
      state.notice = `Import termine: ${imported.length} skill(s)`;
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-skill]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedSkillId = button.dataset.skill ?? "";
      state.selectedGroupId = "";
      state.actionsOpen = false;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-group]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedGroupId = button.dataset.group ?? "";
      state.selectedSkillId = "";
      state.actionsOpen = false;
      render();
    });
  });

  document.querySelector<HTMLButtonElement>("#close-panel")?.addEventListener("click", () => {
    state.selectedSkillId = "";
    state.selectedGroupId = "";
    state.actionsOpen = false;
    render();
  });

  document.querySelector<HTMLFormElement>("#add-command-form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    withTask(async () => {
      if (!isTauriRuntime()) {
        const skill: Skill = {
          id: `demo/${String(form.get("name") ?? "command").toLowerCase().replace(/\s+/g, "-")}`,
          name: String(form.get("name") ?? ""),
          description: String(form.get("description") ?? ""),
          sourceType: "command",
          installCommand: String(form.get("command") ?? ""),
          libraryPath: null,
          version: "demo",
          tags: String(form.get("tags") ?? "")
            .split(",")
            .map((tag) => tag.trim())
            .filter(Boolean),
        };
        state.catalog.skills = [...state.catalog.skills, skill];
        state.view = "skills";
        state.selectedSkillId = skill.id;
        state.commandModalOpen = false;
        state.notice = `Skill ajoute en apercu: ${skill.name}`;
        return;
      }

      const skill = await invoke<Skill>("add_command_skill", {
        input: {
          name: String(form.get("name") ?? ""),
          description: String(form.get("description") ?? ""),
          command: String(form.get("command") ?? ""),
          tags: String(form.get("tags") ?? "")
            .split(",")
            .map((tag) => tag.trim())
            .filter(Boolean),
        },
      });
      await refreshCatalog();
      state.view = "skills";
      state.selectedSkillId = skill.id;
      state.commandModalOpen = false;
      state.notice = `Skill ajoute: ${skill.name}`;
    });
  });

  document.querySelector<HTMLButtonElement>("#install-selected")?.addEventListener("click", () => {
    state.view = "install";
    state.installMode = "skill";
    state.installReference = state.selectedSkillId;
    state.selectedSkillId = "";
    render();
  });

  document.querySelector<HTMLButtonElement>("#export-selected")?.addEventListener("click", () => {
    const skill = selectedSkill();
    if (!skill) return;
    withTask(async () => {
      const outputDir = await chooseDirectory();
      if (!outputDir) return;
      const path = await invoke<string>("export_skill", {
        skillRef: skill.id,
        outputDir,
        overwrite: true,
      });
      state.notice = `Export vers ${path}`;
    });
  });

  document.querySelector<HTMLButtonElement>("#delete-selected")?.addEventListener("click", () => {
    const skill = selectedSkill();
    if (!skill) return;
    if (!confirm(`Supprimer ${skill.name} du catalogue ?`)) return;
    withTask(async () => {
      if (!isTauriRuntime()) {
        state.catalog.skills = state.catalog.skills.filter((item) => item.id !== skill.id);
        state.selectedSkillId = "";
        state.notice = `Skill supprime en apercu: ${skill.name}`;
        return;
      }
      await invoke<Skill>("delete_skill", { skillRef: skill.id });
      state.selectedSkillId = "";
      await refreshCatalog();
      state.notice = `Skill supprime: ${skill.name}`;
    });
  });

  document.querySelector<HTMLFormElement>("#create-group-form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    withTask(async () => {
      if (!isTauriRuntime()) {
        const group: SkillGroup = {
          id: `demo/${String(form.get("name") ?? "group").toLowerCase().replace(/\s+/g, "-")}`,
          name: String(form.get("name") ?? ""),
          description: String(form.get("description") ?? "") || null,
          skillIds: [],
        };
        state.catalog.groups = [...state.catalog.groups, group];
        state.view = "groups";
        state.selectedGroupId = group.id;
        state.groupModalOpen = false;
        state.notice = `Groupe cree en apercu: ${group.name}`;
        return;
      }
      const group = await invoke<SkillGroup>("create_group", {
        name: String(form.get("name") ?? ""),
        description: String(form.get("description") ?? "") || null,
      });
      await refreshCatalog();
      state.view = "groups";
      state.selectedGroupId = group.id;
      state.groupModalOpen = false;
      state.notice = `Groupe cree: ${group.name}`;
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-toggle-skill]").forEach((button) => {
    button.addEventListener("click", () => {
      const group = selectedGroup();
      const skillRef = button.dataset.toggleSkill;
      if (!group || !skillRef) return;
      const included = group.skillIds.includes(skillRef);
      withTask(async () => {
        if (!isTauriRuntime()) {
          group.skillIds = included
            ? group.skillIds.filter((id) => id !== skillRef)
            : [...group.skillIds, skillRef];
          state.selectedGroupId = group.id;
          return;
        }
        await invoke<SkillGroup>(included ? "group_remove_skill" : "group_add_skill", {
          groupRef: group.id,
          skillRef,
        });
        await refreshCatalog();
        state.selectedGroupId = group.id;
      });
    });
  });

  document.querySelector<HTMLButtonElement>("#install-group")?.addEventListener("click", () => {
    state.view = "install";
    state.installMode = "group";
    state.installReference = state.selectedGroupId;
    state.selectedGroupId = "";
    render();
  });

  document.querySelector<HTMLButtonElement>("#delete-group")?.addEventListener("click", () => {
    const group = selectedGroup();
    if (!group) return;
    if (!confirm(`Supprimer le groupe ${group.name} ?`)) return;
    withTask(async () => {
      if (!isTauriRuntime()) {
        state.catalog.groups = state.catalog.groups.filter((item) => item.id !== group.id);
        state.selectedGroupId = "";
        state.notice = `Groupe supprime en apercu: ${group.name}`;
        return;
      }
      await invoke<SkillGroup>("delete_group", { groupRef: group.id });
      state.selectedGroupId = "";
      await refreshCatalog();
      state.notice = `Groupe supprime: ${group.name}`;
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-install-mode]").forEach((button) => {
    button.addEventListener("click", () => {
      state.installMode = button.dataset.installMode as InstallMode;
      state.installReference =
        state.installMode === "skill"
          ? state.catalog.skills[0]?.id ?? ""
          : state.catalog.groups[0]?.id ?? "";
      render();
    });
  });

  document.querySelector<HTMLButtonElement>("#choose-project")?.addEventListener("click", () => {
    withTask(async () => {
      const path = await chooseDirectory();
      if (path) state.installProject = path;
    });
  });

  document.querySelector<HTMLButtonElement>("#choose-target")?.addEventListener("click", () => {
    withTask(async () => {
      const path = await chooseDirectory();
      if (path) state.installTarget = path;
    });
  });

  document.querySelector<HTMLFormElement>("#install-form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    state.installReference = String(form.get("reference") ?? "");
    state.installProject = String(form.get("project") ?? "");
    state.installTarget = String(form.get("target") ?? "");
    state.overwrite = form.get("overwrite") === "on";
    withTask(installCurrentSelection);
  });
}

async function installCurrentSelection() {
  if (!isTauriRuntime()) {
    state.notice = "Installation disponible dans la fenetre Tauri.";
    return;
  }

  const reference = state.installReference;
  const projectPath = state.installProject || null;
  const targetPath = state.installTarget || null;
  const previewCommand =
    state.installMode === "skill" ? "preview_skill_commands" : "preview_group_commands";
  const installCommand = state.installMode === "skill" ? "install_skill" : "install_group";
  const refKey = state.installMode === "skill" ? "skillRef" : "groupRef";
  const previews = await invoke<InstallCommandPreview[]>(previewCommand, {
    [refKey]: reference,
  });

  let executeCommands = false;
  if (previews.length) {
    const commands = previews
      .map((preview) => `${preview.skillName}\n${preview.command}`)
      .join("\n\n");
    executeCommands = confirm(`Executer ces commandes d'installation ?\n\n${commands}`);
    if (!executeCommands) {
      state.notice = "Installation annulee avant execution des commandes.";
      return;
    }
  }

  const outcome = await invoke<InstallationOutcome>(installCommand, {
    [refKey]: reference,
    projectPath,
    targetPath,
    overwrite: state.overwrite,
    executeCommands,
  });
  state.notice = `${outcome.installedSkills.length} element(s) installe(s) vers ${outcome.targetDir}`;
}

refreshCatalog()
  .catch((error) => {
    state.error = String(error);
  })
  .finally(render);
