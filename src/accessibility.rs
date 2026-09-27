use crate::app::{Action as UiAction, EditMode, Modal, State};
use accesskit::{
    Action, ActionData, ActionRequest, Affine, Live, Node, NodeId, Rect, Role, TreeId, TreeInfo,
    TreeUpdate,
};
const ROOT: NodeId = NodeId(0);
const CANVAS: NodeId = NodeId(9000);
const STATUS: NodeId = NodeId(9001);
const CONTENT: NodeId = NodeId(9002);
pub fn tree(s: &State, scale: f64) -> TreeUpdate {
    let mut root = Node::new(Role::Window);
    root.set_label(s.title());
    root.set_bounds(Rect::new(0., 0., s.width as f64, s.height as f64));
    root.set_transform(Affine::scale(scale));
    let mut children = Vec::new();
    let mut nodes = Vec::new();
    for (i, h) in s.hits.iter().enumerate() {
        let id = NodeId(i as u64 + 10);
        let input = matches!(h.action, UiAction::Input(_));
        let mut node = Node::new(if input { Role::TextInput } else { Role::Button });
        node.set_label(match h.action {
            UiAction::GuideAbove => if s.editor.document.guides.above_characters {"Guides devant les caractères. Activer pour les placer derrière."} else {"Guides derrière les caractères et devant la référence. Activer pour les placer devant."}.into(),
            UiAction::GuideRecolorAll => "Appliquer la couleur de guide choisie à tous les traits existants. Annulable.".into(),
            UiAction::GuideOpacity(d) => if d > 0 {
                "Augmenter l’opacité des guides"
            } else {
                "Réduire l’opacité des guides"
            }
            .into(),
            UiAction::GuideWidth(d) => if d > 0 {
                "Augmenter l’épaisseur du pinceau de guides"
            } else {
                "Réduire l’épaisseur du pinceau de guides"
            }
            .into(),
            UiAction::RecolorSize(d) => if d > 0 {
                "Agrandir la brosse de recoloration"
            } else {
                "Réduire la brosse de recoloration"
            }
            .into(),
            UiAction::ThemeColor(t) => {
                let c = t.color(&s.theme());
                format!(
                    "{} : {:02X}{:02X}{:02X}. Modifier la couleur.",
                    t.label(),
                    c[0],
                    c[1],
                    c[2]
                )
            }
            UiAction::ShaderPreviewZoom(true) => "Zoomer dans l’aperçu des shaders".into(),
            UiAction::ShaderPreviewZoom(false) => "Dézoomer dans l’aperçu des shaders".into(),
            UiAction::ShaderPreviewFit => "Ajuster et recentrer l’aperçu des shaders".into(),
            UiAction::ShaderPreviewActual => "Aperçu des shaders à 100 pour cent".into(),
            _ => h.label.clone(),
        });
        node.set_bounds(Rect::new(
            h.rect.x as f64,
            h.rect.y as f64,
            (h.rect.x + h.rect.w) as f64,
            (h.rect.y + h.rect.h) as f64,
        ));
        node.add_action(Action::Focus);
        node.add_action(Action::Click);
        if let UiAction::Input(index) = h.action {
            node.add_action(Action::SetValue);
            let v = match &s.modal {
                Some(Modal::New { width, height } | Modal::Resize { width, height }) => {
                    if index == 0 {
                        width.as_str()
                    } else {
                        height.as_str()
                    }
                }
                Some(Modal::Color { value, .. }) => value.as_str(),
                _ => "",
            };
            node.set_value(v);
        }
        nodes.push((id, node));
        children.push(id);
    }
    if s.modal.is_none() {
        let mut node = Node::new(Role::Pane);
        let c = s
            .editor
            .document
            .get(s.cursor.0, s.cursor.1)
            .unwrap_or_default();
        node.set_label(format!("Canevas {} par {}. Cellule {}, {}. Glyphe {}. Outil {}. Flèches pour déplacer, Entrée pour appliquer.",s.editor.document.width,s.editor.document.height,s.cursor.0,s.cursor.1,ditto::font::character(c.glyph),if s.edit_mode==EditMode::Keyboard {"Clavier : touche = glyphe et avance"}else{s.tool.name()}));
        node.set_bounds(Rect::new(
            s.canvas.x as f64,
            s.canvas.y as f64,
            (s.canvas.x + s.canvas.w) as f64,
            (s.canvas.y + s.canvas.h) as f64,
        ));
        node.add_action(Action::Focus);
        nodes.push((CANVAS, node));
        children.push(CANVAS);
    }
    if let Some(modal) = &s.modal {
        let mut n = Node::new(Role::Label);
        let value=match modal{Modal::Guides=>"Calque de guides. Tracer librement, gommer des traits, régler couleur, épaisseur, opacité, visibilité et position devant ou derrière les caractères. Recolorer tous les traits avec la couleur choisie. Les guides sont enregistrés avec le projet et exclus des exports. Cmd ou Ctrl Z annule.".into(),Modal::Settings=>"Réglages de l’application. Choisir un thème ou personnaliser chaque couleur. Aperçu immédiat. Enregistrer conserve les préférences ; Annuler rétablit le thème précédent.".into(),Modal::Shaders=>"Shaders. Aperçu avant et après. Molette pour zoomer, glisser pour déplacer. Plus et moins pour zoomer, zéro pour ajuster, un pour 100 pour cent, Alt et flèches pour déplacer la vue. Pile appliquée de haut en bas, paramètres annulables et enregistrés avec le projet. Les exports PNG incluent les effets. Tab pour parcourir les réglages, Entrée pour agir, Échap pour fermer.".into(),Modal::Text{content}=>format!("Texte à copier : {content}"),Modal::Charsets=>"Choisir un charset. Touches A à Z et rangée des chiffres sans Maj ni Option. Braille organisé en quatre hauteurs et motifs espacés.".into(),Modal::Help=>"Aide. Cmd ou Ctrl et flèches haut/bas : banque précédente/suivante du charset. Flèches : curseur. Entrée : appliquer. F6 : canevas. Tab : contrôles. Cmd ou Ctrl S : enregistrer. Z : annuler. Shift Z : rétablir. C, X, V : copier, couper, coller. Shift C : aperçu texte. Espace et glisser : déplacer la vue. Alt et flèches : déplacer la vue au clavier. Échap : annuler le geste.".into(),Modal::New{..}=>"Nouveau document. Choisir un préréglage ou saisir les dimensions.".into(),Modal::Resize{..}=>"Redimensionnement. Le contenu est conservé depuis le coin supérieur gauche ; le reste sera recadré. Cette action est annulable.".into(),Modal::Loss{..}=>"Modifications non enregistrées. Enregistrer avant de continuer ?".into(),Modal::Recovery=>"Un brouillon de récupération a été retrouvé.".into(),Modal::Export{..}=>"Export PNG : dessin seul, sans référence ni aides.".into(),Modal::Color{..}=>"Couleur RVB en six chiffres hexadécimaux.".into()};
        n.set_value(value);
        nodes.push((CONTENT, n));
        children.push(CONTENT);
    }
    let mut status = Node::new(Role::Label);
    status.set_value(s.status.clone());
    status.set_live(Live::Polite);
    nodes.push((STATUS, status));
    children.push(STATUS);
    root.set_children(children);
    nodes.push((ROOT, root));
    let focus = s
        .focus
        .map(|i| NodeId(i as u64 + 10))
        .unwrap_or(if s.modal.is_some() { ROOT } else { CANVAS });
    TreeUpdate {
        nodes,
        tree: Some(TreeInfo::new(ROOT)),
        tree_id: TreeId::ROOT,
        focus,
    }
}
pub fn action(s: &mut State, r: ActionRequest) {
    if r.target_node == CANVAS && r.action == Action::Focus {
        s.focus = None;
        s.keyboard_canvas = true;
        return;
    }
    let Some(i) = r.target_node.0.checked_sub(10).map(|v| v as usize) else {
        return;
    };
    let Some(hit) = s.hits.get(i).cloned() else {
        return;
    };
    match r.action {
        Action::Click => s.activate(hit.action),
        Action::Focus => {
            s.focus = Some(i);
            s.keyboard_canvas = false;
            if let UiAction::Input(n) = hit.action {
                s.input = n;
                s.input_replace = true;
            }
        }
        Action::SetValue => {
            if let (UiAction::Input(n), Some(ActionData::Value(value))) = (hit.action, r.data) {
                s.input = n;
                s.input_replace = true;
                s.text_input(&value);
            }
        }
        _ => {}
    }
}
