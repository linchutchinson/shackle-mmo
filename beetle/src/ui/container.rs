use common::math::Rect;
use legion::{
    query::IntoQuery, system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore,
};
use macroquad::window::{screen_height, screen_width};

// TODO Add support for manually sized/positioned UI roots for windowing.
pub struct UIRoot;

pub struct UIContainer {
    pub children: Vec<Entity>,
    pub margin: f32,
    pub gap: f32,
}

#[derive(Copy, Clone)]
pub enum UISize {
    Constant(f32),
    Grow(usize),
}

#[derive(Copy, Clone)]
pub struct UIConstraint {
    width: Option<f32>,
}

impl UIConstraint {
    pub fn width_constraint(width: f32) -> Self {
        Self { width: Some(width) }
    }
}

#[system(for_each)]
pub fn size_ui_root(entity: &Entity, _: &UIRoot, commands: &mut CommandBuffer) {
    let height = screen_height();
    let width = screen_width();
    let rect = Rect::new(0.0, 0.0, width, height);
    commands.add_component(*entity, rect);
}

#[system]
#[read_component(Rect)]
#[read_component(UIContainer)]
#[read_component(UIRoot)]
#[read_component(UISize)]
#[read_component(UIConstraint)]
pub fn layout_ui(world: &mut SubWorld, commands: &mut CommandBuffer) {
    let mut root_query = <(&Rect, &UIContainer, &UIRoot)>::query();

    root_query.iter(world).for_each(|(rect, container, _)| {
        calculate_and_apply_child_ui_sizes(*rect, container, &world, commands);
    });
}

fn calculate_and_apply_child_ui_sizes(
    container_rect: Rect,
    container: &UIContainer,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let size_info: Vec<(UISize, Option<UIConstraint>)> = container
        .children
        .iter()
        .map(|c| {
            if let Ok(entry) = world.entry_ref(*c) {
                let constraint = if let Ok(constraint) = entry.get_component::<UIConstraint>() {
                    Some(*constraint)
                } else {
                    None
                };
                if let Ok(size) = entry.get_component::<UISize>() {
                    (*size, constraint)
                } else {
                    panic!("There is a child ui entity with no defined size!");
                }
            } else {
                panic!("There's a reference to a child ui component that doesn't exist!");
            }
        })
        .collect();

    if size_info.is_empty() {
        // No children. Skip layout.
        return;
    }

    let (constant_used_space, flex_units): (f32, usize) =
        size_info.iter().fold((0.0, 0), |acc, (s, _)| match s {
            //TODO: When vertical constraints are implemented they have to be taken into account here.
            UISize::Constant(s) => (acc.0 + *s, acc.1),
            UISize::Grow(units) => (acc.0, acc.1 + *units),
        });

    let inner_rect = Rect::new(
        container_rect.position.x + container.margin,
        container_rect.position.y + container.margin,
        container_rect.size.x - container.margin * 2.0,
        container_rect.size.y - container.margin * 2.0,
    );
    let flex_space =
        inner_rect.size.y - constant_used_space - (container.gap * (size_info.len() - 1) as f32);
    let flex_unit_size = flex_space / flex_units as f32;

    let mut draw_pos = inner_rect.position.y;

    size_info
        .iter()
        .enumerate()
        .for_each(|(idx, (size, constraint))| {
            let (x, w) = {
                let initial_width = inner_rect.size.x;

                let max_width = if let Some(c) = constraint {
                    c.width
                } else {
                    None
                };

                if let Some(width) = max_width {
                    let constrained_width = initial_width.min(width);
                    let centered_x = inner_rect.position.x + (inner_rect.size.x * 0.5)
                        - (constrained_width * 0.5);
                    (centered_x, constrained_width)
                } else {
                    (inner_rect.position.x, initial_width)
                }
            };
            let h = match size {
                UISize::Constant(s) => *s,
                UISize::Grow(units) => flex_unit_size * *units as f32,
            };

            let child_rect = Rect::new(x, draw_pos, w, h);

            let child_ref = world.entry_ref(container.children[idx]).unwrap();
            if let Ok(child_container) = child_ref.get_component::<UIContainer>() {
                calculate_and_apply_child_ui_sizes(child_rect, child_container, world, commands);
            }

            commands.add_component(container.children[idx], child_rect);

            draw_pos += child_rect.size.y + container.gap;
        });
}
